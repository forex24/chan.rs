use chrono::DateTime;
use chrono::Utc;

use crate::CChanException;
// 已完备
use crate::CEigenFx;
use crate::CSeg;
use crate::CSegConfig;
use crate::ICalcMetric;
use crate::IParent;
use crate::LineType;
use crate::ToHandle;
use crate::{Direction, SegType};

pub struct CSegListChan<T> {
    pub lst: Box<Vec<CSeg<T>>>,
    pub lv: SegType,
    pub config: CSegConfig,
}

impl<T: LineType + IParent + ToHandle + ICalcMetric> CSegListChan<T> {
    /// 创建新的线段列表通道
    ///
    /// # Arguments
    /// * `seg_config` - 线段配置参数
    /// * `lv` - 线段级别
    ///
    /// # Returns
    /// 返回新的CSegListChan实例
    pub fn new(seg_config: CSegConfig, lv: SegType) -> Self {
        Self {
            lst: Box::<Vec<CSeg<T>>>::default(),
            lv,
            config: seg_config,
        }
    }

    /// 清理并弹出最后一个线段，同时重置相关笔的父线段信息
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表，用于检查索引范围
    fn clear_and_pop(&mut self, bi_lst: &[T]) {
        let last_seg = self.lst.last().unwrap();
        // 以下代码是 修复线段变量重置异常bug 新增的
        for bi in &last_seg.bi_list {
            if bi.index() < bi_lst.len() {
                // 这里检查的原因是，如果是虚笔，最后一笔可能失效
                bi.as_mut().set_parent_seg_dir(None);
                bi.as_mut().set_parent_seg_idx(None);
            }
        }
        self.lst.pop();
    }

    // 已完备
    /// 初始化处理，删除末尾不确定的线段
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表，用于重置笔的父线段信息
    fn do_init(&mut self, bi_lst: &[T]) {
        // 删除末尾不确定的线段
        while !self.lst.is_empty() && !self.lst.last().unwrap().is_sure {
            self.clear_and_pop(bi_lst);
        }

        if !self.lst.is_empty() {
            debug_assert!(self.lst.last().unwrap().eigen_fx.is_some());
            debug_assert!(self.lst.last().unwrap().eigen_fx.as_ref().unwrap().ele[2].is_some());

            let last_bi = self.lst.last().unwrap().eigen_fx.as_ref().unwrap().ele[2]
                .as_ref()
                .unwrap()
                .lst
                .last()
                .unwrap();
            if !last_bi.is_sure() {
                {
                    // 如果确定线段的分形的第三元素包含不确定笔，也需要重新算，不然线段分形元素的高低点可能不对
                    // TODO:是否要向该线段包含的笔，设置parent_seg_dir & parent_seg_idx为None
                    self.clear_and_pop(bi_lst);
                }
            }
        }
    }

    // 已完备
    /// 更新线段列表，包括确定线段和处理剩余笔
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表，用于计算新的线段
    pub fn update(&mut self, bi_lst: &[T], clock: &DateTime<Utc>) -> Result<(), CChanException> {
        self.do_init(bi_lst);
        //self.do_init();

        let begin_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst.last().unwrap().end_bi.index() + 1
        };

        self.cal_seg_sure(bi_lst, begin_idx, clock)?;

        self.collect_left_seg(bi_lst, clock)?;
        Ok(())
    }

    /// 计算确定的线段
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    /// * `begin_idx` - 开始计算的笔索引
    fn cal_seg_sure(
        &mut self,
        bi_lst: &[T],
        begin_idx: usize,
        clock: &DateTime<Utc>,
    ) -> Result<(), CChanException> {
        let mut current_idx = begin_idx;

        while current_idx < bi_lst.len() {
            let fx_eigen = self.cal_eigen_fx(bi_lst, current_idx)?;

            if let Some(fx_eigen) = fx_eigen {
                match self.treat_fx_eigen_non_recursive(fx_eigen, bi_lst, clock)? {
                    Some(next_id) => current_idx = next_id,
                    None => break,
                }
            } else {
                break;
            }
        }
        //let fx_eigen = self.cal_eigen_fx(bi_lst, begin_idx)?;
        //if let Some(fx_eigen) = fx_eigen {
        //    self.treat_fx_eigen(fx_eigen, bi_lst, clock)?;
        //}
        //
        Ok(())
    }

    fn treat_fx_eigen_non_recursive(
        &mut self,
        mut fx_eigen: CEigenFx<T>,
        bi_lst: &[T],
        clock: &DateTime<Utc>,
    ) -> Result<Option<usize>, CChanException> {
        let test = fx_eigen.can_be_end(bi_lst);
        let end_bi_idx = fx_eigen.get_peak_bi_idx();

        match test {
            Some(true) | None => {
                // None表示反向分型找到尾部也没找到
                let is_true = test.is_some(); // 如果是正常结束
                if !self.add_new_seg(
                    bi_lst,
                    end_bi_idx,
                    is_true && fx_eigen.all_bi_is_sure(),
                    None,
                    true,
                    "normal",
                    clock,
                )? {
                    // 防止第一根线段的方向与首尾值异常
                    //self.cal_seg_sure(bi_lst, end_bi_idx + 1, clock)?;
                    return Ok(Some(end_bi_idx + 1));
                }

                self.lst.last_mut().unwrap().eigen_fx = Some(fx_eigen);

                if is_true {
                    //self.cal_seg_sure(bi_lst, end_bi_idx + 1, clock)?;
                    return Ok(Some(end_bi_idx + 1));
                }
            }
            Some(false) => {
                //self.cal_seg_sure(bi_lst, fx_eigen.lst[1].index(), clock)?;
                return Ok(Some(fx_eigen.lst[1].index()));
            }
        }

        Ok(None)
    }
    // 99% 完备
    /// 计算特征分型
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    /// * `begin_idx` - 开始计算的笔索引
    ///
    /// # Returns
    /// 返回可能的特征分型
    fn cal_eigen_fx(
        &mut self,
        bi_lst: &[T],
        begin_idx: usize,
    ) -> Result<Option<CEigenFx<T>>, CChanException> {
        let mut up_eigen = CEigenFx::new(Direction::Up, true, self.lv); // 上升线段下降笔
        let mut down_eigen = CEigenFx::new(Direction::Down, true, self.lv); // 下降线段上升笔
        let mut last_seg_dir = if self.lst.is_empty() {
            None
        } else {
            Some(self.lst.last().unwrap().dir)
        };

        for bi in bi_lst.iter().skip(begin_idx) {
            let mut fx_eigen_dir = None;
            match (bi.direction(), last_seg_dir) {
                (Direction::Down, Some(Direction::Down) | None) => {
                    //最后线段的方向不是向上的，意味着当前线段假设方向就是向上的，所以下降笔就是特征序列笔，分型是顶分型
                    if up_eigen.add(bi.to_handle()) {
                        fx_eigen_dir = Some(Direction::Up);
                    }
                }
                (Direction::Up, Some(Direction::Up) | None) => {
                    //最后线段的方向不是向下的，意味着当前线段假设方向就是向下的，所以上升笔就是特征序列笔，分型是底分型
                    if down_eigen.add(bi.to_handle()) {
                        fx_eigen_dir = Some(Direction::Down);
                    }
                }

                _ => {}
            };

            if self.lst.is_empty() {
                // 尝试确定第一段方向，不要以谁先成为分形来决定，反例：US.EVRG
                if up_eigen.ele[1].is_some() && bi.is_down() {
                    last_seg_dir = Some(Direction::Down);
                    down_eigen.clear();
                } else if down_eigen.ele[1].is_some() && bi.is_up() {
                    up_eigen.clear();
                    last_seg_dir = Some(Direction::Up);
                }

                if (up_eigen.ele[1].is_none()
                    && last_seg_dir == Some(Direction::Down)
                    && bi.direction() == Direction::Down)
                    || (down_eigen.ele[1].is_none()
                        && last_seg_dir == Some(Direction::Up)
                        && bi.direction() == Direction::Up)
                {
                    last_seg_dir = None;
                }
            }

            if let Some(dir) = fx_eigen_dir {
                match dir {
                    Direction::Up => return Ok(Some(up_eigen)),
                    Direction::Down => return Ok(Some(down_eigen)),
                }
            }
        }
        Ok(None)
    }

    /// 处理特征分型，确定是否可以构成线段
    ///
    /// # Arguments
    /// * `fx_eigen` - 特征分型
    /// * `bi_lst` - 笔列表
    fn _treat_fx_eigen(
        &mut self,
        mut fx_eigen: CEigenFx<T>,
        bi_lst: &[T],
        clock: &DateTime<Utc>,
    ) -> Result<(), CChanException> {
        let test = fx_eigen.can_be_end(bi_lst);
        let end_bi_idx = fx_eigen.get_peak_bi_idx();

        match test {
            Some(true) | None => {
                // None表示反向分型找到尾部也没找到
                let is_true = test.is_some(); // 如果是正常结束
                if !self.add_new_seg(
                    bi_lst,
                    end_bi_idx,
                    is_true && fx_eigen.all_bi_is_sure(),
                    None,
                    true,
                    "normal",
                    clock,
                )? {
                    // 防止第一根线段的方向与首尾值异常
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1, clock)?;
                    return Ok(());
                }

                self.lst.last_mut().unwrap().eigen_fx = Some(fx_eigen);

                if is_true {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1, clock)?;
                }
            }
            Some(false) => {
                self.cal_seg_sure(bi_lst, fx_eigen.lst[1].index(), clock)?;
            }
        }

        Ok(())
    }
}
