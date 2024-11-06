use crate::{bi, bsp, fx, seg, zs, Opt};

pub async fn parse(opt: &Opt) {
    fx::parse(opt).await;
    bi::parse(opt).await;
    seg::parse(opt).await;
    zs::parse(opt).await;
    bsp::parse(opt).await;
}
