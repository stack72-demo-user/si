use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;
use url::Url;

use super::{PkgResult, SiPkgAttrFuncInput, SiPkgError, SiPkgMapKeyFunc, SiPkgValidation, Source};

use crate::{
    node::{PkgNode, PropChildNode, PropNode},
    FuncUniqueId, PropSpecWidgetKind,
};

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum SiPkgProp<'a> {
    Array {
        name: String,
        default_value: Option<serde_json::Value>,
        func_unique_id: Option<FuncUniqueId>,
        widget_kind: PropSpecWidgetKind,
        widget_options: Option<serde_json::Value>,
        doc_link: Option<Url>,
        hidden: bool,
        hash: Hash,
        source: Source<'a>,
    },
    Boolean {
        name: String,
        default_value: Option<bool>,
        func_unique_id: Option<FuncUniqueId>,
        widget_kind: PropSpecWidgetKind,
        widget_options: Option<serde_json::Value>,
        doc_link: Option<Url>,
        hidden: bool,
        hash: Hash,
        source: Source<'a>,
    },
    Map {
        name: String,
        default_value: Option<serde_json::Value>,
        func_unique_id: Option<FuncUniqueId>,
        widget_kind: PropSpecWidgetKind,
        widget_options: Option<serde_json::Value>,
        doc_link: Option<Url>,
        hidden: bool,
        hash: Hash,
        source: Source<'a>,
    },
    Number {
        name: String,
        default_value: Option<i64>,
        func_unique_id: Option<FuncUniqueId>,
        widget_kind: PropSpecWidgetKind,
        widget_options: Option<serde_json::Value>,
        doc_link: Option<Url>,
        hidden: bool,
        hash: Hash,
        source: Source<'a>,
    },
    Object {
        name: String,
        default_value: Option<serde_json::Value>,
        func_unique_id: Option<FuncUniqueId>,
        widget_kind: PropSpecWidgetKind,
        widget_options: Option<serde_json::Value>,
        doc_link: Option<Url>,
        hidden: bool,
        hash: Hash,
        source: Source<'a>,
    },
    String {
        name: String,
        default_value: Option<String>,
        func_unique_id: Option<FuncUniqueId>,
        widget_kind: PropSpecWidgetKind,
        widget_options: Option<serde_json::Value>,
        hidden: bool,
        doc_link: Option<Url>,
        hash: Hash,
        source: Source<'a>,
    },
}

macro_rules! impl_prop_child_from_graph {
    ($fn_name:ident, PropChildNode::$child_node:ident, $pkg_type:ident) => {
        pub fn $fn_name(&self) -> PkgResult<Vec<$pkg_type>> {
            Ok(match self {
                SiPkgProp::Map { source, .. }
                | SiPkgProp::Array { source, .. }
                | SiPkgProp::String { source, .. }
                | SiPkgProp::Number { source, .. }
                | SiPkgProp::Object { source, .. }
                | SiPkgProp::Boolean { source, .. } => {
                    let mut entries = vec![];
                    if let Some(child_idxs) = source
                        .graph
                        .neighbors_directed(source.node_idx, Outgoing)
                        .find(|node_idx| {
                            matches!(
                                &source.graph[*node_idx].inner(),
                                PkgNode::PropChild(PropChildNode::$child_node)
                            )
                        })
                    {
                        let child_node_idxs: Vec<_> = source
                            .graph
                            .neighbors_directed(child_idxs, Outgoing)
                            .collect();

                        for child_idx in child_node_idxs {
                            entries.push($pkg_type::from_graph(source.graph, child_idx)?);
                        }
                    }

                    entries
                }
            })
        }
    };
}

impl<'a> SiPkgProp<'a> {
    impl_prop_child_from_graph!(validations, PropChildNode::Validations, SiPkgValidation);
    impl_prop_child_from_graph!(inputs, PropChildNode::AttrFuncInputs, SiPkgAttrFuncInput);
    impl_prop_child_from_graph!(map_key_funcs, PropChildNode::MapKeyFuncs, SiPkgMapKeyFunc);

    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let prop_hashed_node = &graph[node_idx];
        let prop_node = match prop_hashed_node.inner() {
            PkgNode::Prop(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::PROP_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let hash = prop_hashed_node.hash();
        let source = Source::new(graph, node_idx);

        Ok(match prop_node {
            PropNode::String {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
            } => Self::String {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
                hash,
                source,
            },
            PropNode::Integer {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
            } => Self::Number {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
                hash,
                source,
            },
            PropNode::Boolean {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
            } => Self::Boolean {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
                hash,
                source,
            },
            PropNode::Map {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
            } => Self::Map {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
                hash,
                source,
            },
            PropNode::Array {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
            } => Self::Array {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,
                doc_link,
                hash,
                source,
            },
            PropNode::Object {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
            } => Self::Object {
                name,
                default_value,
                func_unique_id,
                widget_kind,
                widget_options,
                hidden,

                doc_link,
                hash,
                source,
            },
        })
    }

    pub fn func_unique_id(&self) -> Option<FuncUniqueId> {
        match self {
            Self::String { func_unique_id, .. }
            | Self::Number { func_unique_id, .. }
            | Self::Boolean { func_unique_id, .. }
            | Self::Map { func_unique_id, .. }
            | Self::Array { func_unique_id, .. }
            | Self::Object { func_unique_id, .. } => *func_unique_id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::String { name, .. }
            | Self::Number { name, .. }
            | Self::Boolean { name, .. }
            | Self::Map { name, .. }
            | Self::Array { name, .. }
            | Self::Object { name, .. } => name,
        }
    }

    pub fn hash(&self) -> Hash {
        match self {
            Self::String { hash, .. }
            | Self::Number { hash, .. }
            | Self::Boolean { hash, .. }
            | Self::Map { hash, .. }
            | Self::Array { hash, .. }
            | Self::Object { hash, .. } => *hash,
        }
    }

    pub fn source(&self) -> &Source<'a> {
        match self {
            Self::String { source, .. }
            | Self::Number { source, .. }
            | Self::Boolean { source, .. }
            | Self::Map { source, .. }
            | Self::Array { source, .. }
            | Self::Object { source, .. } => source,
        }
    }
}
