use crate::Bi::Bi::CBi;

use crate::Combiner::KLine_Combiner::CKLineCombiner;
use crate::Common::types::SharedCell;
use crate::Common::CEnum::{BiDir, FxType};

pub struct CEigen {
    pub inner: CKLineCombiner<CBi>,
    pub gap: bool,
}

impl CEigen {
    pub fn new(bi: SharedCell<CBi>, dir: BiDir) -> Self {
        CEigen {
            inner: CKLineCombiner::new(bi, dir),
            gap: false,
        }
    }

    pub fn update_fx(
        &mut self,
        _pre: &CEigen,
        _next: &CEigen,
        exclude_included: bool,
        allow_top_equal: Option<i32>,
    ) {
        self.inner
            .update_fx(&_pre.inner, &_next.inner, exclude_included, allow_top_equal);
        if (self.inner.fx() == FxType::Top && _pre.inner.high() < self.inner.low())
            || (self.inner.fx() == FxType::Bottom && _pre.inner.low() > self.inner.high())
        {
            self.gap = true;
        }
    }

    pub fn get_peak_bi_idx(&self) -> i32 {
        assert!(self.inner.fx() != FxType::Unknown);
        let BiDir = self.inner.lst()[0].borrow().dir;
        if BiDir == BiDir::Up {
            // 下降线段
            self.inner.get_peak_klu(false).borrow().idx - 1
        } else {
            self.inner.get_peak_klu(true).borrow().idx - 1
        }
    }
}

impl std::fmt::Display for CEigen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}~{} gap={} fx={:?}",
            self.inner.lst()[0].borrow().idx,
            self.inner.lst().last().unwrap().borrow().idx,
            self.gap,
            self.inner.fx()
        )
    }
}

// Implement Deref and DerefMut to allow CEigen to be used like CKLineCombiner
impl std::ops::Deref for CEigen {
    type Target = CKLineCombiner<CBi>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for CEigen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
