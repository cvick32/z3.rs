use std::{cell::RefCell, rc::Rc};

use egg::{Analysis, Language};
use log::debug;

use crate::egg_utils::{DefaultCostFunction, RecExprRoot};

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
    L: egg::Language + DefaultCostFunction + std::fmt::Display,
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
        debug!("======>");
        debug!("applying {}", rewrite.name);
        for m in &matches {
            if let Some(cow_ast) = &m.ast {
                let subst = &m.substs[0];
                debug!("cur sub: {:?}", subst);
                // transform &Cow<T> -> &T
                let ast = cow_ast.as_ref();
                // construct a new term by instantiating variables in the pattern ast with terms
                // from the substitution.

                let new_lhs: egg::RecExpr<_> = unpatternify(reify_pattern_ast(ast, egraph, subst));

                if let Some(applier_ast) = rewrite.applier.get_pattern_ast() {
                    let new_rhs: egg::RecExpr<_> =
                        unpatternify(reify_pattern_ast(applier_ast, egraph, subst));
                    let rhs_eclass = egraph.lookup_expr(&new_rhs);
                    // the eclass that we would have inserted from this pattern
                    // would cause a union from `rhs_eclass` to `eclass`. This means it
                    // is creating an equality that wouldn't otherwise be in the
                    // e-graph. This is a conflict, so we record the rule instantiation
                    // here.
                    if Some(m.eclass) != rhs_eclass {
                        debug!("FOUND VIOLATION");
                        debug!("{} => {}", new_lhs.pretty(80), new_rhs.pretty(80));
                        self.instantiations
                            .borrow_mut()
                            .push(format!("(= {} {})", new_lhs, new_rhs));
                    }
                }
            }
        }
        // let n = self
        //     .inner
        //     .apply_rewrite(iteration, egraph, rewrite, matches);
        debug!("<======");
        // we don't actually want to apply the rewrite, because it would be a violation
        0
    }
}

/// We want to replace all the variables in the pattern with terms extracted from
/// the egraph. We do this by calling `join_recexprs` on the root of the pattern
/// ast. For enodes, we want to just return them as is. However, we have to build it
/// fresh, so that the ids work out correctly. For patterns, we call
/// `find_best_variable_substitution` which uses egraph extraction to find the best
/// term.
fn reify_pattern_ast<L, N>(
    pattern: &egg::PatternAst<L>,
    egraph: &egg::EGraph<L, N>,
    subst: &egg::Subst,
) -> egg::PatternAst<L>
where
    L: egg::Language + DefaultCostFunction + std::fmt::Display,
    N: egg::Analysis<L>,
{
    if pattern.as_ref().len() == 1 {
        let node = &pattern.as_ref()[0];
        match node {
            x @ egg::ENodeOrVar::ENode(_) => vec![x.clone()].into(),
            egg::ENodeOrVar::Var(var) => {
                let eclass = &egraph[*subst.get(*var).unwrap()];
                find_best_variable_substitution(egraph, eclass)
            }
        }
    } else {
        pattern
            .root()
            .clone()
            .join_recexprs(|id| match pattern[id].clone() {
                x @ egg::ENodeOrVar::ENode(_) => {
                    if x.is_leaf() {
                        vec![x].into()
                    } else {
                        reify_pattern_ast(&x.build_recexpr(|id| pattern[id].clone()), egraph, subst)
                    }
                }
                egg::ENodeOrVar::Var(var) => {
                    let eclass = &egraph[*subst.get(var).unwrap()];
                    find_best_variable_substitution(egraph, eclass)
                }
            })
    }
}

fn unpatternify<L: egg::Language + std::fmt::Display>(
    pattern: egg::PatternAst<L>,
) -> egg::RecExpr<L> {
    pattern
        .as_ref()
        .iter()
        .map(|node| match node {
            egg::ENodeOrVar::ENode(node) => node.clone(),
            egg::ENodeOrVar::Var(_) => panic!("Can't unpatternify vars"),
        })
        .collect::<Vec<_>>()
        .into()
}

/// TODO: This function should iterate over the nodes in the eclass and choose the variable
/// that has the highest score w.r.t some ranking function. I know there's some notion of
/// ranking that's built into egg, so maybe we can pre-compute this inside the EClass itself
/// and just return `max()` here.
fn find_best_variable_substitution<L, N>(
    egraph: &egg::EGraph<L, N>,
    eclass: &egg::EClass<L, <N as Analysis<L>>::Data>,
) -> egg::PatternAst<L>
where
    L: egg::Language + DefaultCostFunction + std::fmt::Display,
    N: egg::Analysis<L>,
{
    let extractor = egg::Extractor::new(egraph, L::cost_function());
    let (cost, expr) = extractor.find_best(eclass.id);
    debug!(
        "    extraction: {} -> {} (cost: {cost:?})",
        eclass.id,
        expr.pretty(80)
    );
    // wrap everything in an ENodeOrVar so that it still counts as an egg::PatternAst
    expr.as_ref()
        .iter()
        .cloned()
        .map(egg::ENodeOrVar::ENode)
        .collect::<Vec<_>>()
        .into()
}
