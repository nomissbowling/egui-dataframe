#![doc(html_root_url = "https://docs.rs/egui-dataframe/0.3.3")]
//! egui dataframe
//!

use eframe::{self, egui::*};
use egui_grid::GridBuilder;
use egui_extras::{Size, TableBuilder, Column};
use polars::prelude::{DataFrame, Schema}; // , AnyValue, Field, DataType

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
  /// slice Color32 to Vec Option Color32
  pub fn opt(v: &[Color32]) -> Vec<Option<Color32>> {
    v.iter().map(|c| Some(*c)).collect::<Vec<_>>()
  }

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

/// DecoFunc
pub type DecoFunc<'a> = &'a mut dyn FnMut(&Decorator, &mut Ui, &str,
  usize, usize) -> bool;

/// DecoFs
pub struct DecoFs<'a> {
  /// fncs
  pub fncs: (DecoFunc<'a>, DecoFunc<'a>)
}

/// DecoFs
impl<'a> DecoFs<'a> {
  /// default
  pub fn default(d: &Decorator, ui: &mut Ui, tx: &str,
    _ri: usize, _ci: usize) -> bool {
    let _resp_painter = d.disp(ui, tx);
    true
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
  pub fn disp<'a>(&'a self, ui: &'a mut Ui, f: &mut DecoFs, df: &DataFrame,
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
          let d = if self.deco.len() == 0 { &self.default_deco.0 } else {
            match &self.deco[i] {
            (None, _) => &self.default_deco.0,
            (Some(deco_head), _) => deco_head
            }
          };
          f.fncs.0(d, ui, &tx, 0, i);
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
              let d = if self.deco.len() == 0 { &self.default_deco.1 } else {
                match &self.deco[i] {
                (_, None) => &self.default_deco.1,
                (_, Some(deco_body)) => deco_body
                }
              };
              f.fncs.1(d, ui, &tx, ri, i);
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
  pub fn grid<'a>(&'a self, ui: &'a mut Ui, f: &mut DecoFs, df: &DataFrame,
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
              let d = if self.deco.len() == 0 { &self.default_deco.1 } else {
                match &self.deco[i] {
                (_, None) => &self.default_deco.1,
                (_, Some(deco_body)) => deco_body
                }
              };
              f.fncs.1(d, ui, &tx, j, i);
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
    assert_eq!(Decorator::opt(&[Color32::RED, Color32::GREEN]),
      [Some(Color32::RED), Some(Color32::GREEN)]);
  }
}