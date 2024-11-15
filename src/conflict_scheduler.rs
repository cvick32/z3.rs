use std::{cell::RefCell, rc::Rc};

use egg::Language;

#[derive(Clone)]
pub struct ConflictScheduler<S> {
    inner: S,
    /// TODO: use RecExpr instead of String
    /// Keep track of rule instantiations that caused conflicts. We use an
    /// `Rc<RefCell<...>>` here because the scheduler isn't public on `egg::Runner`. So
    /// in order to be able to get data out of the scheduler after a saturation run, we
    /// need to use interior mutability.
    instantiations: Rc<RefCell<Vec<String>>>,
}

impl<S> ConflictScheduler<S> {
    pub fn new(scheduler: S) -> Self {
        Self {
            inner: scheduler,
            instantiations: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn instantiations(&self) -> Rc<RefCell<Vec<String>>> {
        Rc::clone(&self.instantiations)
    }
}

impl<S, L, N> egg::RewriteScheduler<L, N> for ConflictScheduler<S>
where
    S: egg::RewriteScheduler<L, N>,
    L: egg::Language + std::fmt::Display,
    N: egg::Analysis<L>,
{
    fn can_stop(&mut self, iteration: usize) -> bool {
        self.inner.can_stop(iteration)
    }

    fn search_rewrite<'a>(
        &mut self,
        iteration: usize,
        egraph: &egg::EGraph<L, N>,
        rewrite: &'a egg::Rewrite<L, N>,
    ) -> Vec<egg::SearchMatches<'a, L>> {
        self.inner.search_rewrite(iteration, egraph, rewrite)
    }

    fn apply_rewrite(
        &mut self,
        _iteration: usize,
        egraph: &mut egg::EGraph<L, N>,
        rewrite: &egg::Rewrite<L, N>,
        matches: Vec<egg::SearchMatches<L>>,
    ) -> usize {
        println!("======>");
        for m in &matches {
            println!("{m:#?}");
            if let Some(ast) = &m.ast {
                let subst = &m.substs[0];
                let new: egg::RecExpr<_> = ast
                    .as_ref()
                    .as_ref()
                    .iter()
                    .map(|node| match node {
                        egg::ENodeOrVar::ENode(node) => node.clone(),
                        egg::ENodeOrVar::Var(var) => {
                            // TODO: handle all found substs
                            let eclass = &egraph[*subst.get(*var).unwrap()];
                            eclass.nodes[0].clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .into();
                // let slice = ast.as_ref().as_ref();
                // let root = &slice[slice.len() - 1];
                // let new = root.build_recexpr(|id| {
                //     match &ast[id] {

                //     }
                // });

                if let Some(applier_ast) = rewrite.applier.get_pattern_ast() {
                    let new_rhs: egg::RecExpr<_> = applier_ast
                        .as_ref()
                        .iter()
                        .map(|node| match node {
                            egg::ENodeOrVar::ENode(node) => node.clone(),
                            egg::ENodeOrVar::Var(var) => {
                                let eclass = &egraph[*subst.get(*var).unwrap()];
                                eclass.nodes[0].clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .into();

                    println!("{applier_ast:#?}");
                    println!("{} => {}", new.pretty(80), new_rhs.pretty(80));

                    let blah = egraph.lookup_expr(&new_rhs);
                    // the eclass that we would have inserted from this pattern
                    // would cause a union from `blah` to `eclass`. This means it
                    // is creating an equality that wouldn't otherwise be in the
                    // e-graph. This is a conflict, so we record the rule instantiation
                    // here.
                    if Some(m.eclass) != blah {
                        self.instantiations.borrow_mut().push(new.pretty(80));
                    }
                    println!("{blah:?}, {}", Some(m.eclass) == blah);
                }
            }
        }

        // let n = self
        //     .inner
        //     .apply_rewrite(iteration, egraph, rewrite, matches);
        println!("applying {}", rewrite.name);
        println!("<=====");
        0
    }
}
