import pytest
from datetime import datetime, timedelta
from czsc import Analyzer

def create_kline(time_str, open_price, high_price, low_price, close_price, volume=1000.0):
    """创建一个K线数据"""
    return {
        "time": time_str,
        "open": float(open_price),
        "high": float(high_price),
        "low": float(low_price),
        "close": float(close_price),
        "volume": float(volume)
    }

def generate_test_klines(count=100, start_price=100.0):
    """生成测试用的K线数据"""
    base_time = datetime(2023, 1, 1)
    klines = []
    current_price = start_price
    
    for i in range(count):
        time_str = (base_time + timedelta(minutes=i)).strftime("%Y-%m-%d %H:%M:%S")
        # 生成一些波动的价格
        high = current_price * (1 + 0.01 * (i % 3))
        low = current_price * (1 - 0.01 * (i % 2))
        close = (high + low) / 2
        
        klines.append(create_kline(
            time_str=time_str,
            open_price=current_price,
            high_price=high,
            low_price=low,
            close_price=close
        ))
        current_price = close
    
    return klines

class TestAnalyzer:
    """Analyzer类的测试用例"""
    
    def test_analyzer_creation(self):
        """测试Analyzer的创建"""
        # 测试默认配置创建
        analyzer = Analyzer(step_calculation=1)
        assert analyzer is not None
        
        # 测试自定义配置创建
        custom_config = {
            "bi_algo": "new",
            "bi_min_len": 7,
            "seg_algo": "chan",
        }
        analyzer = Analyzer(step_calculation=1, config=custom_config)
        assert analyzer is not None

    def test_add_single_kline(self):
        """测试添加单个K线"""
        analyzer = Analyzer(step_calculation=1)
        kline = create_kline(
            time_str="2023-01-01 00:00:00",
            open_price=100.0,
            high_price=105.0,
            low_price=95.0,
            close_price=103.0
        )
        analyzer.add_k(kline)
        
        # 验证K线是否被正确添加
        candle_list = analyzer.candle_list
        assert len(candle_list) == 1
        assert candle_list[0]["close"] == 103.0

    def test_add_multiple_klines(self):
        """测试添加多个K线"""
        analyzer = Analyzer(step_calculation=1)
        klines = generate_test_klines(count=10)
        
        for kline in klines:
            analyzer.add_k(kline)
        
        # 验证K线列表
        assert len(analyzer.candle_list) == 10
        
        # 验证bar列表
        assert len(analyzer.bar_list) > 0

    def test_bi_generation(self):
        """测试笔的生成"""
        analyzer = Analyzer(step_calculation=1)
        klines = generate_test_klines(count=100)  # 生成足够多的K线以形成笔
        
        for kline in klines:
            analyzer.add_k(kline)
        
        # 验证是否生成了笔
        assert len(analyzer.bi_list) > 0

    def test_invalid_kline_data(self):
        """测试无效K线数据的处理"""
        analyzer = Analyzer(step_calculation=1)
        
        # 测试缺少必要字段的K线
        invalid_kline = {"time": "2023-01-01 00:00:00", "open": 100.0}
        with pytest.raises(Exception):
            analyzer.add_k(invalid_kline)
        
        # 测试价格为负的K线
        invalid_kline = create_kline(
            time_str="2023-01-01 00:00:00",
            open_price=-100.0,
            high_price=105.0,
            low_price=95.0,
            close_price=103.0
        )
        with pytest.raises(Exception):
            analyzer.add_k(invalid_kline)

    def test_property_access(self):
        """测试属性访问"""
        analyzer = Analyzer(step_calculation=1)
        klines = generate_test_klines(count=50)
        
        for kline in klines:
            analyzer.add_k(kline)
        
        # 测试所有属性是否可以访问
        assert hasattr(analyzer, "bi_list")
        assert hasattr(analyzer, "seg_list")
        assert hasattr(analyzer, "candle_list")
        assert hasattr(analyzer, "bar_list")
        assert hasattr(analyzer, "bi_bsp_list")
        assert hasattr(analyzer, "bi_zs_list")

    @pytest.mark.parametrize("step_calculation", [1, 2, 5])
    def test_different_step_calculations(self, step_calculation):
        """测试不同的step_calculation值"""
        analyzer = Analyzer(step_calculation=step_calculation)
        klines = generate_test_klines(count=20)
        
        for kline in klines:
            analyzer.add_k(kline)
        
        assert len(analyzer.candle_list) == 20

def test_real_market_scenario():
    """测试真实市场场景"""
    analyzer = Analyzer(step_calculation=1)
    
    # 模拟一个上涨-下跌-上涨的场景
    base_price = 100.0
    prices = []
    
    # 上涨阶段
    for i in range(10):
        prices.append(base_price + i * 2)
    
    # 下跌阶段
    for i in range(10):
        prices.append(base_price + 20 - i * 2)
    
    # 再次上涨
    for i in range(10):
        prices.append(base_price + i * 2)
    
    # 创建并添加K线
    base_time = datetime(2023, 1, 1)
    for i, price in enumerate(prices):
        time_str = (base_time + timedelta(minutes=i)).strftime("%Y-%m-%d %H:%M:%S")
        kline = create_kline(
            time_str=time_str,
            open_price=price,
            high_price=price * 1.01,
            low_price=price * 0.99,
            close_price=price
        )
        analyzer.add_k(kline)
    
    # 验证是否正确识别了趋势
    assert len(analyzer.bi_list) > 0
    assert len(analyzer.seg_list) > 0

if __name__ == "__main__":
    pytest.main([__file__])
