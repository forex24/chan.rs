use crate::{Bar, Candle, FxType, Handle, KlineDir};

pub struct CandleList {
    #[allow(clippy::box_collection)]
    pub candle_list: Box<Vec<Candle>>,
}

impl CandleList {
    /// 创建新的K线列表实例
    /// 
    /// # Returns
    /// 返回一个新的CandleList实例，内部容器初始容量为1,024,000
    pub fn new() -> Self {
        CandleList {
            candle_list: Box::new(Vec::with_capacity(1_024_000)),
        }
    }

    /// 更新K线
    /// 
    /// # Arguments
    /// * `bar` - 新的Bar数据
    /// 
    /// # Returns
    /// 返回是否生成了新的K线
    /// 
    /// 如果是第一根K线，直接添加；否则尝试合并到最后一根K线，
    /// 如果不能合并则创建新的K线，并检查分型
    pub fn update_candle(&mut self, bar: Handle<Bar>) -> bool {
        if self.candle_list.len() == 0 {
            let candle = Candle::new(&self.candle_list, bar, 0, KlineDir::Up);
            self.candle_list.push(candle);
        } else {
            let _dir = self.candle_list.last_mut().unwrap().try_add(bar);

            if _dir != KlineDir::Combine {
                let candle = Candle::new(&self.candle_list, bar, self.candle_list.len(), _dir);
                self.candle_list.push(candle);

                if self.candle_list.len() >= 3 {
                    let index = self.candle_list.len() - 2;
                    self.candle_list[index].fx_type = self.check_fx();
                }

                return true;
            }
        }
        false
    }

    /// 检查分型
    /// 
    /// # Returns
    /// 返回分型类型（顶分型、底分型或未知）
    /// 
    /// 通过比较倒数三根K线的高低点，判断是否形成顶分型或底分型
    fn check_fx(&mut self) -> FxType {
        let len = self.candle_list.len();
        let _cur = &self.candle_list[len - 2];
        let _pre = &self.candle_list[len - 3];
        let _next = &self.candle_list[len - 1];
        if _pre.high < _cur.high
            && _next.high < _cur.high
            && _pre.low < _cur.low
            && _next.low < _cur.low
        {
            return FxType::Top;
        }

        if _pre.high > _cur.high
            && _next.high > _cur.high
            && _pre.low > _cur.low
            && _next.low > _cur.low
        {
            return FxType::Bottom;
        }
        FxType::Unknown
    }
}

impl Default for CandleList {
    /// 实现Default trait，使用new()作为默认构造方法
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for CandleList {
    type Target = Box<Vec<Candle>>;

    /// 实现Deref trait，允许直接访问内部的K线列表
    fn deref(&self) -> &Self::Target {
        &self.candle_list
    }
}

impl std::ops::DerefMut for CandleList {
    /// 实现DerefMut trait，允许直接修改内部的K线列表
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.candle_list
    }
}
