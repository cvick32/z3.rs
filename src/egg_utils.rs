/// Trait for saturating an egraph with the array axioms. This hides the details of
/// needing to create a runner every time you want to saturate a set of rules on an egraph.
pub trait Saturate {
    fn saturate(&mut self) -> Vec<String>;
}

pub trait DefaultCostFunction: egg::Language {
    fn cost_function() -> impl CompareCost<Self>;
}

pub trait CompareCost<L: egg::Language>: egg::CostFunction<L> {
    fn lt(&self, x: Self::Cost, y: u32) -> bool;
}

pub trait RecExprRoot<L> {
    fn root(&self) -> &L;
}

impl<L> RecExprRoot<L> for egg::RecExpr<L> {
    fn root(&self) -> &L {
        let ast_nodes = self.as_ref();
        &ast_nodes[ast_nodes.len() - 1]
    }
}
