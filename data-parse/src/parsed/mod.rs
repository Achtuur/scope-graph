use std::{
    collections::HashMap,
    fs::OpenOptions,
    hash::Hash,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{
    ParseResult,
    raw::{JavaType, RawEdge, RawScopeGraph, RefType},
};

mod label;
mod scope;

pub use label::*;
pub use scope::*;

// https://stackoverflow.com/questions/51276896/how-do-i-use-serde-to-serialize-a-hashmap-with-structs-as-keys-to-json
pub mod vectorize {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::iter::FromIterator;

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        let container: Vec<_> = target.into_iter().collect();
        serde::Serialize::serialize(&container, ser)
    }

    pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        let container: Vec<_> = serde::Deserialize::deserialize(des)?;
        Ok(T::from_iter(container))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ScopeData {
    Ref(ParsedScope),
    ClassOrMethod(String, ParsedScope),
    /// Scope that is combined
    Combined,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedScopeGraph {
    #[serde(with = "vectorize")]
    pub scopes: HashMap<ParsedScope, ScopeData>,
    pub edges: Vec<ParsedEdge>,
    pub labels: Vec<JavaLabel>,
}

impl TryFrom<RawScopeGraph> for ParsedScopeGraph {
    type Error = crate::ParseError;

    fn try_from(raw: RawScopeGraph) -> ParseResult<Self> {
        println!(
            "raw.data: {0:?}",
            raw.data
                .get("#/./org/apache/commons/csv/ExtendedBufferedReader.jav-d_581-0")
        );

        let scopes = raw
            .data
            .into_iter()
            .map(|(scope_key, data)| {
                let s = ParsedScope::from_str(&scope_key)?;
                let d = match data.into_data() {
                    Some(JavaType::Scope(_)) => ScopeData::None, // scope declaration is not data, but good
                    Some(JavaType::Ref(RefType::ScopeRef(raw_scope))) => {
                        ScopeData::Ref(ParsedScope::from(raw_scope.arg0))
                    }
                    Some(JavaType::Ref(RefType::MethodOrClass(m))) => {
                        let (id, scope) = m.into_id_scope();
                        ScopeData::ClassOrMethod(id, ParsedScope::from(scope))
                    }
                    Some(JavaType::MethodOrClass(m)) => {
                        let (id, scope) = m.into_id_scope();
                        ScopeData::ClassOrMethod(id, ParsedScope::from(scope))
                    }
                    Some(_) => ScopeData::None,
                    None => ScopeData::None,
                };
                Ok((s, d))
            })
            .collect::<ParseResult<HashMap<_, _>>>()?;

        let edges = raw
            .edges
            .into_iter()
            .flat_map(|(key, edge)| ParsedEdge::from_raw(key, RawEdge::Head(edge)).unwrap())
            .collect::<Vec<_>>();

        let labels = raw
            .labels
            .into_iter()
            .map(ParsedLabel::from)
            .map(JavaLabel::try_from)
            .collect::<ParseResult<Vec<_>>>()?;

        Ok(Self {
            scopes,
            edges,
            labels,
        })
    }
}

impl ParsedScopeGraph {
    pub fn from_file<P: AsRef<Path>>(path: P) -> ParseResult<Self> {
        match Self::read_cache(&path) {
            Ok(graph) => return Ok(graph),
            Err(e) => {
                println!("Cache read failed: {}", e);
                let _ = std::fs::remove_file(Self::cache_path(&path));
            }
        }

        println!("Cache doesn't exist, reading raw file, this can take a while...");
        let file = OpenOptions::new().read(true).open(&path)?;
        let mut buf = BufReader::new(file);
        let timer = std::time::Instant::now();
        let mut deserializer = serde_json::Deserializer::from_reader(&mut buf);
        deserializer.disable_recursion_limit();
        let json: RawScopeGraph = Deserialize::deserialize(&mut deserializer)?;
        println!("Deserialization took: {:?}", timer.elapsed());
        let graph = ParsedScopeGraph::try_from(json)?;
        if let Err(e) = graph.write_cache(&path) {
            println!("Failed to write cache: {}", e);
        }
        Ok(graph)
    }

    pub fn filter_scopes(&mut self, filter: fn(&ParsedScope) -> bool) {
        self.edges
            .retain(|edge| filter(&edge.from) || filter(&edge.to));
        self.filter_scopes_without_edges();
    }

    pub fn filter_scope_by_edge_labels<F>(&mut self, filter: F)
    where
        F: Fn(&ParsedScope, Option<&ParsedEdge>, Option<&ParsedEdge>) -> bool,
    {
        self.scopes.retain(|s, _| {
            let incoming_edges = self.edges.iter().filter(|e| &e.to == s).collect::<Vec<_>>();
            let outgoing_edges = self.edges.iter().filter(|e| &e.from == s).collect::<Vec<_>>();

            if incoming_edges.is_empty() {
                outgoing_edges.iter().any(|e| filter(s, None, Some(e)))
            }
            else if outgoing_edges.is_empty() {
                incoming_edges.iter().any(|e| filter(s, Some(e), None))
            }
            else {
                for e_in in incoming_edges {
                    for e_out in &outgoing_edges {
                        if filter(s, Some(e_in), Some(e_out)) {
                            return true;
                        }
                    }
                }
                false
            }

        });

        // for scope in remove_scopes.iter() {
        //     self.scopes.remove(scope);
        // }

        self.edges.retain(|e| {
            self.scopes.contains_key(&e.from) && self.scopes.contains_key(&e.to)
        });
    }

    pub fn filter_edges(&mut self, filter: fn(&ParsedEdge) -> bool) {
        self.edges.retain(filter);
        self.filter_scopes_without_edges();
    }

    fn filter_scopes_without_edges(&mut self) {
        self.scopes.retain(|scope, _| {
            self.edges
                .iter()
                .any(|e| e.from == *scope || e.to == *scope)
        });
    }

    /// Combines scopes that refer to each other.
    ///
    /// Ie if a scope exists that declares the class and another that contains the class body,
    /// they are combined.
    pub fn combine_scopes(&mut self) {
        let mut from_edge_map = HashMap::new();
        let mut to_edge_map = HashMap::new();
        for e in &mut self.edges {
            from_edge_map
                .entry(e.from.clone())
                .or_insert_with(Vec::new)
                .push(&mut e.from);
            to_edge_map
                .entry(e.to.clone())
                .or_insert_with(Vec::new)
                .push(&mut e.to);
        }

        let mut new_scopes = Vec::new();
        let mut remove_scopes = Vec::new();

        #[derive(PartialEq, Eq)]
        struct ScopeRef<'a> {
            scope: &'a ParsedScope,
            name: Option<&'a str>,
        }

        impl Hash for ScopeRef<'_> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.scope.hash(state);
            }
        }

        // get <referenced scope: existing scope> pairs

        let ref_scopes = self
            .scopes
            .iter()
            .filter_map(|(s, d)| match d {
                // ScopeData::Ref(rs) => Some((rs, None, s)),
                ScopeData::Ref(rs) => Some((rs, None, s)),
                // ScopeData::ClassOrMethod(m_name, rs) => Some((ScopeRef { scope: rs, name: Some(m_name)}, s)),
                ScopeData::ClassOrMethod(m_name, rs) => Some((rs, Some(m_name), s)),
                _ => None,
            })
            // some really dumb shit so I can make sure that every `ScopeRef` has a name
            .fold(HashMap::new(), |mut acc, (referenced, name, orig)| {
                let entry = acc.entry(referenced).or_insert((None, Vec::new()));
                if let Some(n) = name {
                    entry.0 = Some(n.as_str());
                }
                entry.1.push(orig);
                acc
            })
            .into_iter()
            .map(|(referenced, (name, orig))| {
                (
                    ScopeRef {
                        scope: referenced,
                        name,
                    },
                    orig,
                )
            });

        for (referenced, orig) in ref_scopes {
            let name = match &referenced.name {
                Some(n) => format!("{}-{}", referenced.scope.name, n),
                None => referenced.scope.name.to_string(),
            };
            let new_scope = ParsedScope::new(name, referenced.scope.resource.clone());

            for edge_scope in from_edge_map
                .get_mut(referenced.scope)
                .unwrap_or(&mut Vec::new())
            {
                **edge_scope = new_scope.clone();
            }

            for edge_scope in to_edge_map
                .get_mut(referenced.scope)
                .unwrap_or(&mut Vec::new())
            {
                **edge_scope = new_scope.clone();
            }

            for s in orig {
                for edge_scope in from_edge_map.get_mut(s).unwrap_or(&mut Vec::new()) {
                    **edge_scope = new_scope.clone();
                }

                for edge_scope in to_edge_map.get_mut(s).unwrap_or(&mut Vec::new()) {
                    **edge_scope = new_scope.clone();
                }
                remove_scopes.push(s.clone());
            }
            remove_scopes.push(referenced.scope.clone());
            new_scopes.push(new_scope);
        }

        for old in remove_scopes {
            self.scopes.remove(&old);
        }
        for new in new_scopes {
            self.scopes.insert(new, ScopeData::Combined);
        }
    }

    pub fn to_cosmograph_csv<P: AsRef<Path>>(&self, path: P) -> ParseResult<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)?;
        let mut buf = BufWriter::new(file);
        buf.write_all(b"source;target;strength;label;color;width\n")?;

        let mut occ = HashMap::<_, usize>::new();

        for e in &self.edges {
            let from = e.from.name();
            let to = e.to.name();

            occ.entry(&e.from).and_modify(|c| *c += 1).or_insert(0);
            occ.entry(&e.to).and_modify(|c| *c += 1).or_insert(0);

            let _ = buf.write(
                format!(
                    "{};{};{};{};{};{}\n",
                    from,
                    to,
                    e.label.cosmo_value(),
                    e.label,
                    e.label.cosmo_color(),
                    e.label.cosmo_width()
                )
                .as_bytes(),
            )?;
        }

        let meta_fname = format!(
            "{}-meta.csv",
            path.as_ref().file_name().unwrap().to_str().unwrap()
        );
        let meta_path = path.as_ref().parent().map(|p| p.join(meta_fname)).unwrap();

        let meta_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(meta_path)?;
        let mut meta_buf = BufWriter::new(meta_file);
        meta_buf.write_all(b"id;color;size\n")?;
        for (s, d) in &self.scopes {
            let id = s.name();
            let color = s.cosmo_color();
            let n_edges = occ.get(s).unwrap_or(&0);
            let size = match s.is_data() {
                true => 1,
                false => 5 + 2 * n_edges,
            };
            let _ = meta_buf.write(format!("{};{};{}\n", id, color, size).as_bytes())?;
        }

        buf.flush()?;
        meta_buf.flush()?;
        Ok(())
    }

    fn read_cache<P: AsRef<Path>>(path: P) -> ParseResult<Self> {
        let path = Self::cache_path(path);
        let file = OpenOptions::new().read(true).open(&path)?;
        let mut buf = BufReader::new(file);
        let timer = std::time::Instant::now();
        let json: Self = serde_json::from_reader(&mut buf)?;
        println!("Deserialization from cache took: {:?}", timer.elapsed());
        Ok(json)
    }

    fn write_cache<P: AsRef<Path>>(&self, path: P) -> ParseResult<()> {
        println!("Caching graph to: {:?}", path.as_ref());
        let path = Self::cache_path(path);
        // if path.exists() {
        //     println!("Cache file already exists at: {:?}", path);
        //     return Ok(());
        // }
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)?;
        let mut buf = BufWriter::new(file);
        serde_json::to_writer(&mut buf, self)?;
        Ok(())
    }

    fn cache_path<P: AsRef<Path>>(path: P) -> PathBuf {
        let pathbuf = PathBuf::from(path.as_ref());
        let file_name = pathbuf
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default();
        PathBuf::from(format!("/tmp/{}", file_name))
    }
}
