use crate::builtins;
use crate::context::AnalyzerContext;
use crate::db::Analysis;
use crate::errors::TypeError;
use crate::namespace::items::{
    self, DepGraph, DepGraphWrapper, DepLocality, Function, FunctionId, Item, TraitId, TypeDef,
};
use crate::namespace::scopes::ItemScope;
use crate::namespace::types::{self, Contract};
use crate::traversal::types::type_desc;
use crate::AnalyzerDb;
use fe_parser::ast;
use indexmap::map::{Entry, IndexMap};
use smol_str::SmolStr;
use std::rc::Rc;
use std::str::FromStr;

pub fn trait_type(db: &dyn AnalyzerDb, trait_: TraitId) -> Rc<types::Trait> {
    Rc::new(types::Trait {
        name: trait_.name(db),
        id: trait_,
    })
}


// pub fn trait_dependency_graph(
//     db: &dyn AnalyzerDb,
//     trait_: TraitId,
// ) -> Analysis<DepGraphWrapper> {
//     // A trait depends on the types of its fields and on everything they depend on.
//     // It *does not* depend on its public functions; those will only be part of
//     // the broader dependency graph if they're in the call graph of some public
//     // contract function.

//     let scope = ItemScope::new(db, trait_.module(db));
//     let root = Item::Type(TypeDef::Trait(trait_));
//     let fields = trait_
//         .fields(db)
//         .values()
//         .filter_map(|field| match field.typ(db).ok()? {
//             FixedSize::Contract(Contract { id, .. }) => Some((
//                 root,
//                 Item::Type(TypeDef::Contract(id)),
//                 DepLocality::External,
//             )),
//             // Not possible yet, but it will be soon
//             FixedSize::Trait(Trait { id, .. }) => {
//                 Some((root, Item::Type(TypeDef::Trait(id)), DepLocality::Local))
//             }
//             _ => None,
//         })
//         .collect::<Vec<_>>();

//     let mut graph = DepGraph::from_edges(fields.iter());
//     for (_, item, _) in fields {
//         if let Some(subgraph) = item.dependency_graph(db) {
//             graph.extend(subgraph.all_edges())
//         }
//     }

//     Analysis::new(
//         DepGraphWrapper(Rc::new(graph)),
//         scope.diagnostics.take().into(),
//     )
// }

pub fn trait_cycle(
    db: &dyn AnalyzerDb,
    _cycle: &[String],
    trait_: &TraitId,
) -> Analysis<DepGraphWrapper> {
    let mut scope = ItemScope::new(db, trait_.module(db));
    let trait_data = &trait_.data(db).ast;
    scope.error(
        &format!("recursive trait `{}`", trait_data.name()),
        trait_data.kind.name.span,
        &format!(
            "trait `{}` has infinite size due to recursive definition",
            trait_data.name(),
        ),
    );

    Analysis::new(
        DepGraphWrapper(Rc::new(DepGraph::new())),
        scope.diagnostics.take().into(),
    )
}
