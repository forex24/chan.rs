use czsc::{CBi, CSeg, Indexable, LineType, Point};

/*pub fn seg_to_point(seg_list: &[CSeg<CBi>]) -> (Vec<Point>, Vec<Point>) {
    let mut sure_points: Vec<Point> = Vec::new();
    let mut unsure_points: Vec<Point> = Vec::new();
    let mut last_sure_pos = None;
    for seg in seg_list.iter().rev() {
        if seg.is_sure {
            last_sure_pos = Some(seg.index());
            break;
        }
    }
    for seg in seg_list {
        let start_point = Point::new(
            seg.start_bi
                //.begin_klc
                //.get_peak_klu(seg.start_bi.is_down())
                .get_begin_klu()
                .time,
            seg.start_bi.get_begin_val(),
        );
        let end_point = Point::new(
            seg.end_bi /* .end_klc*/
                .get_end_klu()
                .time, //get_peak_klu(seg.start_bi.is_down()).time,
            seg.end_bi.get_end_val(),
        );
        if last_sure_pos.is_some() && seg.index() <= last_sure_pos.unwrap() {
            sure_points.push(start_point);
            sure_points.push(end_point);
        } else {
            unsure_points.push(start_point);
            unsure_points.push(end_point);
        }
    }
    (sure_points, unsure_points)
}*/

pub fn bi_to_point(bi_list: &[Box<CBi>]) -> (Vec<Point>, Vec<Point>) {
    let mut sure_points: Vec<Point> = Vec::new();
    let mut unsure_points: Vec<Point> = Vec::new();
    let mut last_sure_pos = None;
    for bi in bi_list.iter().rev() {
        if bi.is_sure {
            last_sure_pos = Some(bi.index());
            break;
        }
    }

    for bi in bi_list {
        let start_point = Point::new(
            bi.begin_klc.get_begin_klu().time, //get_peak_klu(pen.is_down()).time,
            bi.get_begin_val(),
        );
        let end_point = Point::new(
            bi.end_klc.get_end_klu().time, //get_peak_klu(pen.is_down()).time,
            bi.get_end_val(),
        );
        if last_sure_pos.is_some() && bi.index() <= last_sure_pos.unwrap() {
            sure_points.push(start_point);
            sure_points.push(end_point);
        } else {
            unsure_points.push(start_point);
            unsure_points.push(end_point);
        }
    }
    (sure_points, unsure_points)
}

pub fn seg_to_point<T: LineType>(seg_list: &[Box<CSeg<T>>]) -> (Vec<Point>, Vec<Point>) {
    let mut sure_points: Vec<Point> = Vec::new();
    let mut unsure_points: Vec<Point> = Vec::new();
    let mut last_sure_pos = None;
    for seg in seg_list.iter().rev() {
        if seg.is_sure {
            last_sure_pos = Some(seg.index());
            break;
        }
    }
    for seg in seg_list {
        let start_point = Point::new(
            seg.start_bi.get_begin_klu().time,
            seg.start_bi.get_begin_val(),
        );
        let end_point = Point::new(
            seg.end_bi.get_end_klu().time, //get_peak_klu(seg.start_bi.is_down()).time,
            seg.end_bi.get_end_val(),
        );
        if last_sure_pos.is_some() && seg.index() <= last_sure_pos.unwrap() {
            sure_points.push(start_point);
            sure_points.push(end_point);
        } else {
            unsure_points.push(start_point);
            unsure_points.push(end_point);
        }
    }
    (sure_points, unsure_points)
}
