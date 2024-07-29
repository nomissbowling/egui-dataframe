#![doc(html_root_url = "https://docs.rs/egui-dataframe/0.1.1")]
//! egui dataframe
//!

// use std::error::Error;
use eframe::{self, egui::*};
use egui_grid::GridBuilder;
use egui_extras::{Size, TableBuilder, Column};
use polars::prelude::{DataFrame, AnyValue, Schema, Field, DataType};

/// to anyvalue from value and datatype
/// let a = to_any!(3, DataType::UInt64);
/// let b = to_any!("X", DataType::Utf8);
#[macro_export]
macro_rules! to_any {
  ($v: expr, DataType::Null) => { AnyValue::Null };
  // Date: feature dtype-date
  // Time: feature dtype-date
  // DataType:: DateTime, Duration, Categorical, List, Object, Struct
  //   feature dtype-datetime -duration -categorical -array
  // AnyValue:: Enum, Array, Decimal, xxxOwned, etc
  ($v: expr, DataType:: $t: ident) => { AnyValue::$t($v) }
}
// pub to_any;

/// row schema from vec AnyValue (column names are column_0, column_1, ...)
/// - let schema = Schema::from(&row);
pub fn row_schema(row: Vec<AnyValue<'_>>) -> polars::frame::row::Row {
  polars::frame::row::Row::new(row)
}

/// row fields from vec (&amp;str, DataType) (set with column names)
/// - let schema = Schema::from_iter(fields);
pub fn row_fields(desc: Vec<(&str, DataType)>) -> Vec<Field> {
  desc.into_iter().map(|(s, t)| Field::new(s, t)).collect()
}

/// named fields from DataFrame
pub fn named_fields(df: &DataFrame, n: Vec<&str>) -> Vec<Field> {
  let t = df.dtypes();
  row_fields(n.into_iter().enumerate().map(|(i, s)|
    (s, t[i].clone())).collect())
}

/// named schema from DataFrame
/// - same as df.schema() after column names are set by df.set_column_names()
/// - otherwise df.schema() returns names as column_0, column_1, ...
pub fn named_schema(df: &DataFrame, n: Vec<&str>) -> Schema {
  Schema::from_iter(named_fields(&df, n))
}

/// Decorator
#[derive(Debug, Clone)]
pub struct Decorator {
  /// sz: Vec2
  pub sz: Vec2,
  /// sense: Sense
  pub sense: Sense,
  /// cols: vec![bgr, bgc, fgc]
  pub cols: Vec<Option<Color32>>,
  /// align: Align2
  pub align: Align2,
  /// ofs: Vec2
  pub ofs: Vec2,
  /// fontid: FontId
  pub fontid: FontId
}

/// Decorator
impl Decorator {
  /// constructor
  pub fn new(sz: Vec2, sense: Sense, cols: Vec<Option<Color32>>,
    align: Align2, ofs: Vec2, fontid: FontId) -> Self {
    Decorator{sz, sense, cols, align, ofs, fontid}
  }

  /// paint text as painter.text allocated from ui
  /// - ui: &amp;mut Ui
  /// - txt: &str
  /// - result: (Response, Painter)
  pub fn disp(&self, ui: &mut Ui, txt: &str) -> Option<(Response, Painter)> {
    let (bgr, bgc, fgc) = (self.cols[0], self.cols[1], self.cols[2]);
    if let Some(fgc) = fgc {
      let (resp, painter) = ui.allocate_painter(self.sz, self.sense);
      let rc = resp.rect;
      // rc.max = rc.min + sz; // when calc mut rc (same with resp.rect)
      if let Some(bgr) = bgr {
        if let Some(bgc) = bgc {
          painter.rect(rc, 0.0, bgc, Stroke{width: 1.0, color: bgr});
        }
      }
      painter.text(rc.min + self.ofs, self.align, txt,
        self.fontid.clone(), fgc);
      Some((resp, painter))
    }else{
      ui.label(RichText::new(txt)
        .size(self.fontid.size).family(self.fontid.family.clone()));
      None
    }
  }
}

/// DFDesc
#[derive(Debug, Clone)]
pub struct DFDesc {
  /// default deco
  pub default_deco: (Decorator, Decorator),
  /// deco (head, body=column)
  pub deco: Vec<(Option<Decorator>, Option<Decorator>)>,
  /// schema
  pub schema: Schema
}

/// DFDesc
impl DFDesc {
  /// constructor
  pub fn new(default_deco: (Decorator, Decorator), schema: Schema) -> Self {
    let n = schema.len();
    DFDesc{default_deco,
      deco: Vec::<(Option<Decorator>, Option<Decorator>)>::with_capacity(n),
      schema}
  }

  /// all from (pipeline)
  pub fn all_from(mut self,
    deco: Vec<(Option<Decorator>, Option<Decorator>)>) -> Self {
    self.deco = deco; // deco.into_iter().map(|hb| self.push(hb));
    self
  }

  /// all default (pipeline)
  pub fn all_default(mut self) -> Self {
    for _i in 0..self.schema.len() { self.push((None, None)) }
    self
  }

  /// push deco (head, body=column)
  pub fn push(&mut self, deco: (Option<Decorator>, Option<Decorator>)) {
    self.deco.push(deco);
  }

  /// display dataframe to ui (TableBuilder)
  pub fn disp<'a>(&'a self, ui: &'a mut Ui, df: &DataFrame,
    height_head: f32, height_body: f32, resizable: bool, striped: bool,
    vscroll: bool) {
    let (nrows, ncols) = (df.height(), df.width());
    let cols = df.get_column_names();
    TableBuilder::new(ui).columns(Column::auto().resizable(resizable), ncols)
    .resizable(resizable)
    .striped(striped)
    .vscroll(vscroll)
    .header(height_head, |mut header| {
      for (i, head) in cols.iter().enumerate() {
        header.col(|ui| {
          let tx = format!("{}", head);
          let _r_p = (match &self.deco[i] {
          (None, _) => &self.default_deco.0,
          (Some(deco_head), _) => deco_head
          }).disp(ui, &tx);
          // ui.label(&tx); // ui.heading(&tx);
        });
      }
    })
    .body(|body| {
      body.rows(height_body, nrows, |ri, mut row| {
        for (i, col) in cols.iter().enumerate() {
          row.col(|ui| {
            if let Ok(column) = &df.column(col) {
//              if let Ok(value) = column.get(ri) {
              let value = column.get(ri);
              let tx = format!("{}", value);
              let _r_p = (match &self.deco[i] {
              (_, None) => &self.default_deco.1,
              (_, Some(deco_body)) => deco_body
              }).disp(ui, &tx);
              // ui.label(&tx);
//              }
            }
          });
        }
      });
    });
  }

  /// display grid to ui (GridBuilder)
  /// - ma: &amp;style::Margin::[same(f32) or symmetric(f32, f32)]
  pub fn grid<'a>(&'a self, ui: &'a mut Ui, df: &DataFrame,
    width: f32, height: f32, ts: &TextStyle,
    sp: &(f32, f32), ma: &style::Margin) {
    let (nrows, ncols) = (df.height(), df.width());
    ui.style_mut().override_text_style = Some(ts.clone());
    let mut gb = GridBuilder::new().spacing(sp.0, sp.1).clip(true);
    gb = (0..nrows).into_iter().fold(gb, |gb, _i|
      (0..ncols).into_iter().fold(gb.new_row(Size::exact(height)), |gb, _j|
        gb.cell(Size::exact(width)).with_margin(ma.clone())
        // gb.cell(Size::remainder()).with_margin(ma.clone())
      )
    );
    gb.show(ui, |mut grid| {
      let cols = df.get_column_names();
      for j in 0..nrows {
        for (i, col) in cols.iter().enumerate() {
          // grid.empty();
          grid.cell(|ui| {
            if let Ok(column) = &df.column(col) {
//              if let Ok(value) = column.get(j) {
              let value = column.get(j);
              let tx = format!("{}", value);
              let _r_p = (match &self.deco[i] {
              (_, None) => &self.default_deco.1,
              (_, Some(deco_body)) => deco_body
              }).disp(ui, &tx);
              // ui.label(&tx);
//              }
            }
          });
        }
      }
    });
  }
}

/// tests
#[cfg(test)]
mod tests {
  use super::*;

  /// [-- --nocapture] [-- --show-output]
  /// https://github.com/rust-lang/rust/pull/103681
  /// https://github.com/rust-lang/rust/issues/104053
  /// -- --test-threads=[0|1]
  /// RUST_TEST_THREADS=[0|1]
  /// https://github.com/rust-lang/rust/pull/107396
  /// fn dispatch_to_ui_thread&<lt;R, F&gt;>(f: F) -&gt;> R where
  ///   F: FnOnce() -&gt;> R + Send, R: Send
  /// #[test(flavour = "static_thread")]
  #[test]
  fn test_dataframe() {
    let a = to_any!(3, DataType::UInt64);
    assert_eq!(a, AnyValue::UInt64(3));
    assert_eq!(a.dtype(), DataType::UInt64);
    let b = to_any!("A", DataType::Utf8);
    assert_eq!(b, AnyValue::Utf8("A"));
    assert_eq!(b.dtype(), DataType::Utf8);
    let c = to_any!(4, DataType::Int8);
    assert_eq!(c, AnyValue::Int8(4));
    assert_eq!(c.dtype(), DataType::Int8);
    let d = to_any!(1.5, DataType::Float64);
    assert_eq!(d, AnyValue::Float64(1.5));
    assert_eq!(d.dtype(), DataType::Float64);
    let e = to_any!(true, DataType::Boolean);
    assert_eq!(e, AnyValue::Boolean(true));
    assert_eq!(e.dtype(), DataType::Boolean);
    let f = to_any!(&[255, 0], DataType::Binary);
    assert_eq!(f, AnyValue::Binary(&[255, 0]));
    assert_eq!(f.dtype(), DataType::Binary);
  }
}