use quickcheck::{Arbitrary, Gen};
use serde::{Deserialize, Serialize};

use crate::implementation::{Ctx, Expr, Typ};

use trace::init_depth_var;

init_depth_var!();

pub fn gen_var(ctx: Ctx, t: Typ) -> Vec<usize> {
    ctx.into_iter()
        .enumerate()
        .filter_map(|(i, typ)| if typ == t { Some(i) } else { None })
        .collect()
}

fn extend_ctx(mut ctx: Ctx, t: Typ) -> Ctx {
    ctx.insert(0, t);
    ctx
}

pub fn gen_zero(ctx: Ctx, t: Typ, g: &mut Gen) -> Option<Expr> {
    match t {
        Typ::TBool => Some(Expr::Bool(bool::arbitrary(g))),
        Typ::TFun(box t1, box t2) => {
            let e = gen_zero(extend_ctx(ctx, t1.clone()), t2, g)?;
            Some(Expr::Abs(t1, Box::new(e)))
        }
    }
}



pub fn gen_expr(ctx: Ctx, t: Typ, g: &mut Gen, size: usize) -> Option<Expr> {
    let (ctx1, ctx2) = (ctx.clone(), ctx.clone());
    let (t1, tau) = (t.clone(), t.clone());
    if size == 0 {
        g.backtrack(&[
            (
                1,
                Box::new(move |g: &mut Gen| {
                    let ctx1 = ctx1.clone();
                    let t1 = t1.clone();
                    let vars = gen_var(ctx1, t1);
                    if vars.is_empty() {
                        return None;
                    }
                    g.one_of::<Option<Expr>>(
                        vars.into_iter()
                            .map(|x| {
                                let f: Box<dyn FnOnce(&mut Gen) -> Option<Expr>> =
                                    Box::new(move |_| Some(Expr::Var(x as i32)));
                                f
                            })
                            .collect::<Vec<_>>(),
                    )
                }),
            ),
            (
                1,
                Box::new(move |g: &mut Gen| gen_zero(ctx.clone(), t.clone(), g)),
            ),
        ])
    } else {
        g.backtrack(&[
            (
                1,
                Box::new(move |g: &mut Gen| {
                    let ctx1 = ctx1.clone();
                    let t1 = t1.clone();
                    let vars = gen_var(ctx1, t1);
                    if vars.is_empty() {
                        return None;
                    }
                    g.one_of(
                        vars.into_iter()
                            .map(|x| {
                                let f: Box<dyn FnOnce(&mut Gen) -> Option<Expr>> =
                                    Box::new(move |_| Some(Expr::Var(x as i32)));
                                f
                            })
                            .collect::<Vec<_>>(),
                    )
                }),
            ),
            // (
            //     1,
            //     Box::new(move |g: &mut Gen| {
            //         let ctx1 = ctx.clone();
            //         let t1 = t.clone();

            //         let t2 = gen_typ(g, size);
            //         let e1 = gen_expr(
            //             ctx1.clone(),
            //             Typ::TFun(Box::new(t2.clone()), Box::new(t1)),
            //             g,
            //             size / 2,
            //         )?;
            //         let e2 = gen_expr(ctx1, t2, g, size / 2)?;
            //         Some(Expr::App(Box::new(e1), Box::new(e2)))
            //     }),
            // ),
            (
                1,
                Box::new(move |g: &mut Gen| {
                    let ctx1 = ctx2.clone();
                    let tau = tau.clone();
                    match tau {
                        Typ::TBool => Some(Expr::Bool(bool::arbitrary(g))),
                        Typ::TFun(box t1, box t2) => {
                            let e = gen_expr(extend_ctx(ctx1, t1.clone()), t2.clone(), g, size / 2)?;
                            Some(Expr::Abs(t2, Box::new(e)))
                        }
                    }
                }),
            ),
        ])
    }
}

fn gen_typ(g: &mut Gen, size: usize) -> Typ {
    if size == 0 {
        Typ::TBool
    } else {
        g.frequency(&[
            (1, Box::new(|_| Typ::TBool)),
            (
                size,
                Box::new(move |g| {
                    Typ::TFun(
                        Box::new(gen_typ(g, size / 2)),
                        Box::new(gen_typ(g, size / 2)),
                    )
                }),
            ),
        ])
    }
}

impl Arbitrary for Typ {
    fn arbitrary(g: &mut Gen) -> Self {
        let size = (g.size());
        gen_typ(g, size)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprOpt(pub Option<Expr>);

impl Arbitrary for ExprOpt {
    fn arbitrary(g: &mut Gen) -> Self {
        let typ = Typ::arbitrary(g);
        let ctx: Ctx = vec![];
        let size = g.size();
        Self(gen_expr(ctx, typ, g, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::Expr;
    use quickcheck::quickcheck;

    #[test]
    fn test_gen_zero1() {
        let t = Typ::TBool;
        let ctx: Ctx = vec![];
        let mut g = quickcheck::Gen::new(0);
        let expr = gen_zero(ctx, t, &mut g);
        assert!(expr == Some(Expr::Bool(true)) || expr == Some(Expr::Bool(false)));
    }

    #[test]
    fn test_gen_zero2() {
        let t = Typ::TFun(Box::new(Typ::TBool), Box::new(Typ::TBool));
        let ctx: Ctx = vec![];
        let mut g = quickcheck::Gen::new(1);
        let expr = gen_zero(ctx, t, &mut g);
        assert!(
            expr == Some(Expr::Abs(Typ::TBool, Box::new(Expr::Bool(true))))
                || expr == Some(Expr::Abs(Typ::TBool, Box::new(Expr::Bool(false))))
        );
    }

    #[test]
    fn test_gen_expr1() {
        let t = Typ::TFun(Box::new(Typ::TBool), Box::new(Typ::TBool));
        let ctx: Ctx = vec![];
        let mut g = quickcheck::Gen::new(1);
        let expr = gen_expr(ctx, t, &mut g, 1);
        assert!(expr.is_some());
    }

    #[test]
    fn test_gen_expr2() {
        let t = Typ::TFun(Box::new(Typ::TBool), Box::new(Typ::TBool));
        let ctx: Ctx = vec![
            Typ::TBool,
            Typ::TFun(Box::new(Typ::TBool), Box::new(Typ::TBool)),
        ];
        let mut g = quickcheck::Gen::new(5);
        let expr = gen_expr(ctx, t, &mut g, 5);
        // expr should have a `Var`
    }
}
