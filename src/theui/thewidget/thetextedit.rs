use fontdue::{layout::LayoutSettings, Font};
use unicode_segmentation::UnicodeSegmentation;

use crate::prelude::*;

#[derive(Default, PartialEq)]
pub struct TheCursor {
    pub row: usize,
    pub column: usize,
}

impl TheCursor {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }

    pub fn zero() -> Self {
        Self::default()
    }

    pub fn is_zero(&self) -> bool {
        self.row == 0 && self.column == 0
    }

    pub fn reset(&mut self) {
        self.row = 0;
        self.column = 0;
    }
}

struct TheGlyph {
    parent: char,

    start: usize,
    end: usize,

    x: f32,
    width: usize,
}

struct TheRowInfo {
    top: usize,
    left: usize,
    bottom: usize,
    right: usize,

    baseline: usize,
    glyph_start: usize,
    glyph_end: usize,

    highlights: Option<Vec<(TheColor, usize)>>,
}

#[derive(Default)]
pub struct TheSelection {
    pub start: usize,
    pub end: usize,
}

impl TheSelection {
    pub fn intersects(&self, start: usize, end: usize) -> bool {
        start < self.end && end > self.start
    }

    pub fn is_none(&self) -> bool {
        self.start == self.end
    }

    pub fn reset(&mut self) {
        self.start = 0;
        self.end = 0;
    }
}

// cursor index  0   1   2   3   4
//  glyph index    0   1   2   3
//       cursor  |   |   |   |   |
//         text    a   b   c   \n
pub struct TheTextEditState {
    // Use cursor index
    pub cursor: TheCursor,
    // Linebreak is not stored here
    pub rows: Vec<String>,
    // Use cursor index
    pub selection: TheSelection,
}

impl Default for TheTextEditState {
    fn default() -> Self {
        Self {
            cursor: TheCursor::default(),
            rows: vec![String::default()],
            selection: TheSelection::default(),
        }
    }
}

impl TheTextEditState {
    pub fn cut_text(&mut self) -> String {
        let text = self.get_text(self.selection.start, self.selection.end);
        self.delete_text_by_selection();
        text
    }

    pub fn delete_text(&mut self) -> bool {
        let deleted = if !self.selection.is_none() {
            self.delete_text_by_selection()
        } else {
            self.delete_char_by_cursor()
        };

        if self.rows.is_empty() {
            self.insert_row();
        }

        deleted
    }

    // Position of cursor in cursor index
    pub fn find_cursor_index(&self) -> usize {
        self.find_start_index_of_row(self.cursor.row) + self.cursor.column
    }

    pub fn find_row_col_of_index(&self, index: usize) -> (usize, usize) {
        let row = self.find_row_number_of_index(index);
        let row_start_index = self.find_start_index_of_row(row);
        let col = index - row_start_index;
        (row, col)
    }

    // Row index of glyph index
    // glyph index  0   1   2   3
    //        text  a   b   c   \n
    //         row  0
    // glyph index  4   5   6   7
    //        text  d   e   f   \n
    //         row  1
    pub fn find_row_number_of_index(&self, index: usize) -> usize {
        let mut left = 0;
        let mut right = self.row_count();
        while left < right {
            let row_number = left + (right - left) / 2;
            let (row_start, row_end) = self.find_range_of_row(row_number);

            if index < row_start {
                right = row_number;
            } else if index >= row_end {
                left = row_number + 1;
            } else {
                return row_number;
            }
        }

        self.row_count() - 1
    }

    // Range of row in cursor index
    // cursor index  0   1   2   3   4
    //         text    a   b   c   \n
    //        range  (0, 4)
    // cursor index  4   5   6   7   8
    //         text    d   e   f   \n
    //        range  (4, 8)
    pub fn find_range_of_row(&self, row_number: usize) -> (usize, usize) {
        let start = self.find_start_index_of_row(row_number);
        let end = start + self.row_len(row_number);
        (start, end)
    }

    // Range of selected glyphs within a row
    pub fn find_selected_range_of_row(&self, row_number: usize) -> Option<(usize, usize)> {
        if self.selection.is_none() {
            return None;
        }

        let (start, end) = self.find_range_of_row(row_number);
        if !self.selection.intersects(start, end) {
            return None;
        }

        // Select the linebreak only
        if self.selection.start == end - 1 {
            return Some((end - 1, end));
        }

        let left = self.selection.start.max(start);
        let right = self.selection.end.min(
            // If it's an empty row, we select the linebreak
            // Or if it's the last row
            if start + 1 == end || self.is_last_row(row_number) {
                end
            } else {
                // Eliminate the linebreak if the row is not empty,
                // and it's not the last row
                end - 1
            },
        );
        if left == right {
            None
        } else {
            Some((left, right))
        }
    }

    // Start position of row in cursor index
    pub fn find_start_index_of_row(&self, row_number: usize) -> usize {
        let mut index = 0;
        for i in 0..row_number {
            index += self.row_len(i)
        }
        index
    }

    pub fn insert_char(&mut self, char: char) {
        if !self.selection.is_none() {
            self.delete_text_by_selection();
        }

        let insert_index = self.byte_offset_of_index(self.cursor.row, self.cursor.column);
        self.rows[self.cursor.row].insert(insert_index, char);
        self.move_cursor_right();
    }

    pub fn insert_text(&mut self, text: String) -> (usize, usize) {
        if !self.selection.is_none() {
            self.delete_text_by_selection();
        }

        let start = self.find_cursor_index();
        let glyph_count = text.graphemes(true).count();
        let insert_index = self.byte_offset_of_index(self.cursor.row, self.cursor.column);
        if !text.contains('\n') {
            self.rows[self.cursor.row].insert_str(insert_index, &text);
            self.cursor.column += glyph_count;
            return (start, start + glyph_count);
        }

        let mut rows = text.split('\n');
        let leftover = self.rows[self.cursor.row].split_off(insert_index);
        self.rows[self.cursor.row].insert_str(insert_index, rows.next().unwrap());

        for str in rows {
            self.cursor.row += 1;
            self.rows.insert(self.cursor.row, str.to_owned());
            self.cursor.column = self.glyphs_in_row(self.cursor.row);
        }

        if !leftover.is_empty() {
            let insert_index = self.byte_offset_of_index(self.cursor.row, self.cursor.column);
            self.rows[self.cursor.row].insert_str(insert_index, &leftover);
        }

        (start, start + glyph_count)
    }

    pub fn insert_row(&mut self) {
        if !self.selection.is_none() {
            self.delete_text_by_selection();
        }

        // Insert at current row
        if self.cursor.column == 0 {
            self.rows.insert(self.cursor.row, String::default());
        // Insert at next row
        } else if self.cursor.column >= self.glyphs_in_row(self.cursor.row) {
            self.rows.insert(self.cursor.row + 1, String::default());
        // Insert inside current row
        } else {
            let insert_index = self.byte_offset_of_index(self.cursor.row, self.cursor.column);
            let new_text = self.rows[self.cursor.row].split_off(insert_index);
            self.rows.insert(self.cursor.row + 1, new_text);
        }

        self.cursor.column = 0;
        self.move_cursor_down();
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() || (self.rows.len() == 1 && self.rows[0].is_empty())
    }

    pub fn is_last_row(&self, row_number: usize) -> bool {
        row_number == self.row_count() - 1
    }

    pub fn is_row_all_selected(&self, row_number: usize) -> bool {
        self.find_selected_range_of_row(row_number)
            .map_or(false, |selected_range| {
                let range = self.find_range_of_row(row_number);
                range.0 == selected_range.0 && range.1 == selected_range.1 + 1
            })
    }

    pub fn move_cursor_down(&mut self) -> bool {
        if self.is_last_row(self.cursor.row) {
            return false;
        }

        self.cursor.row += 1;
        self.cursor.column = self.cursor.column.min(self.glyphs_in_row(self.cursor.row));
        true
    }

    pub fn move_cursor_left(&mut self) -> bool {
        if self.cursor.is_zero() {
            return false;
        }

        if self.cursor.column == 0 {
            self.cursor.row -= 1;
            self.cursor.column = self.glyphs_in_row(self.cursor.row);
        } else {
            self.cursor.column -= 1;
        }
        true
    }

    pub fn move_cursor_right(&mut self) -> bool {
        if self.is_last_row(self.cursor.row)
            && self.cursor.column == self.glyphs_in_row(self.cursor.row)
        {
            return false;
        }

        if self.cursor.column == self.glyphs_in_row(self.cursor.row) {
            self.cursor.row += 1;
            self.cursor.column = 0;
        } else {
            self.cursor.column += 1;
        }
        true
    }

    pub fn move_cursor_to_line_end(&mut self) -> bool {
        if self.cursor.column == self.glyphs_in_row(self.cursor.row) {
            return false;
        }

        self.cursor.column = self.glyphs_in_row(self.cursor.row);
        true
    }

    pub fn move_cursor_to_line_start(&mut self) -> bool {
        if self.cursor.column == 0 {
            return false;
        }

        self.cursor.column = 0;
        true
    }

    pub fn move_cursor_up(&mut self) -> bool {
        if self.cursor.row == 0 {
            return false;
        }

        self.cursor.row -= 1;
        self.cursor.column = self.cursor.column.min(self.glyphs_in_row(self.cursor.row));
        true
    }

    pub fn move_lines_down(&mut self) -> bool {
        if self.selection.is_none() {
            self.move_lines(self.cursor.row, self.cursor.row, 1)
        } else {
            self.move_lines(
                self.find_row_number_of_index(self.selection.start),
                self.find_row_number_of_index(self.selection.end),
                1,
            )
        }
    }

    pub fn move_lines_up(&mut self) -> bool {
        if self.selection.is_none() {
            self.move_lines(self.cursor.row, self.cursor.row, -1)
        } else {
            self.move_lines(
                self.find_row_number_of_index(self.selection.start),
                self.find_row_number_of_index(self.selection.end),
                -1,
            )
        }
    }

    pub fn quick_select(&mut self) {
        let text = &self.rows[self.cursor.row];
        let (row_start, row_end) = self.find_range_of_row(self.cursor.row);

        // Cursor is at the end of the row
        if self.cursor.column >= text.len() {
            // Select the linebreak of previous row
            if self.is_last_row(self.cursor.row) {
                if text.is_empty() {
                    self.selection.start = row_start - 1;
                    self.selection.end = row_start;
                    self.move_cursor_left();
                    return;
                }
            // Select the linebreak at the end of row
            } else {
                self.selection.start = row_end - 1;
                self.selection.end = row_end;
                return;
            }
        }

        // Select the empty space
        let col = self.cursor.column.min(text.len() - 1);
        let (start, end) = if text.chars().nth(col).unwrap().is_whitespace() {
            find_range(text, col, |char| !char.is_whitespace())
        }
        // Select a word or the entire row
        else {
            find_range(text, col, |char| char.is_whitespace())
        };

        self.selection.start = row_start + start;
        self.selection.end = row_start + end;

        fn find_range<P>(text: &str, index: usize, predicate: P) -> (usize, usize)
        where
            P: Fn(char) -> bool,
        {
            let start = text[..index]
                .char_indices()
                .rev()
                .find(|&(_, c)| predicate(c))
                .map_or(0, |(i, _)| i + 1);

            let end = text[index + 1..]
                .char_indices()
                .find(|&(_, c)| predicate(c))
                .map_or(text.len(), |(i, _)| index + 1 + i);

            (start, end)
        }
    }

    pub fn reset(&mut self) {
        self.rows = vec![String::default()];
        self.reset_cursor();
        self.reset_selection();
    }

    pub fn reset_cursor(&mut self) {
        self.cursor.reset();
    }

    pub fn reset_selection(&mut self) {
        self.selection.reset();
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn select(&mut self, start: usize, end: usize) {
        self.selection.start = start;
        self.selection.end = end;
    }

    pub fn select_all(&mut self) {
        self.selection.start = 0;
        self.selection.end = self
            .rows
            .iter()
            .enumerate()
            .fold(0, |acc, (i, _)| acc + self.row_len(i));
    }

    pub fn select_row(&mut self) {
        let (start, end) = self.find_range_of_row(self.cursor.row);
        self.select(start, end);
    }

    pub fn set_cursor(&mut self, cursor: TheCursor) {
        self.cursor = cursor;
    }

    pub fn set_text(&mut self, text: String) {
        self.rows = text.split('\n').map(|s| s.to_string()).collect();
    }

    pub fn to_text(&self) -> String {
        self.rows.join("\n")
    }

    fn byte_offset_of_index(&self, row_number: usize, index: usize) -> usize {
        self.rows[row_number]
            .grapheme_indices(true)
            .nth(index)
            .map(|(byte_offset, _)| byte_offset)
            .unwrap_or(self.rows[row_number].len())
    }

    fn delete_char_by_cursor(&mut self) -> bool {
        if self.cursor.is_zero() {
            return false;
        }

        // Delete linebreak and concat with previous row
        if self.cursor.column == 0 {
            self.cursor.column = self.glyphs_in_row(self.cursor.row - 1);
            let text = self.rows.remove(self.cursor.row);
            self.rows[self.cursor.row - 1].push_str(&text);
            self.move_cursor_up();
            return true;
        }

        // Delete normal char
        if self.delete_range_of_row(self.cursor.row, self.cursor.column - 1, self.cursor.column) {
            self.move_cursor_left();
            return true;
        }

        false
    }

    fn delete_range_of_row(&mut self, row_number: usize, start: usize, end: usize) -> bool {
        let left = start.min(end);
        let right = start.max(end).min(self.glyphs_in_row(row_number));
        if left == right {
            return false;
        }

        let left = self.byte_offset_of_index(row_number, left);
        let right = self.byte_offset_of_index(row_number, right);
        let text = &mut self.rows[row_number];
        let remaining = text.split_off(right);
        text.truncate(left);
        text.push_str(&remaining);

        true
    }

    fn delete_text_by_selection(&mut self) -> bool {
        if self.selection.is_none() {
            return false;
        }

        let cursor_index = self.find_cursor_index();
        let start_row = self.find_row_number_of_index(self.selection.start);
        let end_row = self.find_row_number_of_index(self.selection.end);

        // Find selection range of first row,
        // to be used in the future
        let (row_start, row_end) = self.find_range_of_row(start_row);
        let (start, end) = self
            .find_selected_range_of_row(start_row)
            .unwrap_or((row_end, row_end + 1));

        if start_row != end_row {
            // Handle last row
            self.delete_range_of_row(
                end_row,
                0,
                self.selection.end - self.find_start_index_of_row(end_row),
            );
            let text = self.rows.remove(end_row);
            // When only linebreak is selected, manually add a linebreak,
            // so we can delete chars safely later
            if self.selection.end == row_end && self.selection.end - 1 == self.selection.start {
                self.rows[start_row].push('\n');
            }
            self.rows[start_row].push_str(&text);

            // Remove inter rows
            for row_number in (start_row + 1..end_row).rev() {
                self.rows.remove(row_number);
            }
        }

        // Handle first row
        let left = start - row_start;
        let right = end - row_start;
        self.delete_range_of_row(start_row, left, right);

        // Reset cursor
        if cursor_index >= self.selection.start {
            if cursor_index < self.selection.end {
                self.cursor.row = start_row;
                self.cursor.column = left;
            } else {
                let cursor_index = cursor_index - (self.selection.end - self.selection.start);
                let (row, col) = self.find_row_col_of_index(cursor_index);
                self.cursor.row = row;
                self.cursor.column = col;
            }
        }

        self.reset_selection();

        true
    }

    fn get_text(&self, start: usize, end: usize) -> String {
        let (start_row, start_col) = self.find_row_col_of_index(start);
        let (end_row, end_col) = self.find_row_col_of_index(end);

        if start_row == end_row {
            self.rows[start_row][start_col..end_col].to_owned()
        } else {
            let mut text = self.rows[start_row][start_col..].to_owned();
            for row in &self.rows[start_row + 1..end_row] {
                text.push('\n');
                text.push_str(row.as_str());
            }
            text.push('\n');
            text.push_str(&self.rows[end_row][..end_col]);
            text
        }
    }

    fn glyphs_in_row(&self, row_number: usize) -> usize {
        return self.rows[row_number].graphemes(true).count();
    }

    // Inclusive on both end
    fn move_lines(&mut self, start: usize, end: usize, vector: isize) -> bool {
        if vector == 0
            || (start as isize) + vector < 0
            || ((end as isize) + vector).abs().as_usize() >= self.row_count()
        {
            return false;
        }

        if vector < 0 {
            let vector = vector.abs().as_usize();
            if !self.selection.is_none() {
                let row_len = self.glyphs_in_row(start - vector) + 1;
                self.selection.start -= row_len;
                self.selection.end -= row_len;
            }
            for i in start..=end {
                self.rows.swap(i, i - vector);
            }
            self.cursor.row -= vector;
        } else {
            let vector = vector.abs().as_usize();
            if !self.selection.is_none() {
                let row_len = self.glyphs_in_row(end + vector) + 1;
                self.selection.start += row_len;
                self.selection.end += row_len;
            }
            for i in (start..=end).rev() {
                self.rows.swap(i, i + vector);
            }
            self.cursor.row += vector;
        }

        true
    }

    // Length of row in glyphs, linebreak included
    fn row_len(&self, row_number: usize) -> usize {
        // +1 to include the linebreak,
        // except for the last row
        let len = self.glyphs_in_row(row_number);
        if self.is_last_row(row_number) {
            len
        } else {
            len + 1
        }
    }
}

pub struct TheTextRenderer {
    // Dim
    left: usize,
    top: usize,
    width: usize,
    height: usize,

    // Options
    cursor_width: usize,
    cursor_vertical_shrink: usize,
    pub font_size: f32,
    pub padding: (usize, usize, usize, usize), // left top right bottom
    selection_extend: usize,

    // State
    pub actual_size: Vec2<usize>,
    glyphs: Vec<TheGlyph>,
    pub highlighter: Option<Box<dyn TheCodeHighlighterTrait>>,
    row_info: Vec<TheRowInfo>,
    pub scroll_offset: Vec2<usize>,
}

impl Default for TheTextRenderer {
    fn default() -> Self {
        Self {
            left: 0,
            top: 0,
            width: 0,
            height: 0,

            cursor_width: 2,
            cursor_vertical_shrink: 1,
            font_size: 14.0,
            padding: (5, 0, 5, 0),
            selection_extend: 2,

            actual_size: Vec2::zero(),
            glyphs: vec![],
            highlighter: None,
            row_info: vec![],
            scroll_offset: Vec2::zero(),
        }
    }
}

impl TheTextRenderer {
    pub fn dim(&self) -> TheDim {
        TheDim::new(
            (self.left - self.padding.0).as_i32(),
            (self.top - self.padding.1).as_i32(),
            (self.width + self.padding.0 + self.padding.2).as_i32(),
            (self.height + self.padding.1 + self.padding.3).as_i32(),
        )
    }

    pub fn find_cursor(&self, coord: &Vec2<i32>) -> TheCursor {
        let coord = vec2i(
            coord.x + self.scroll_offset.x.as_i32() - self.padding.0.as_i32(),
            coord.y + self.scroll_offset.y.as_i32() - self.padding.1.as_i32(),
        );
        let mut cursor = TheCursor::zero();

        if (coord.x < 0 && coord.y < 0) || self.glyphs.is_empty() {
            // Cursor is at the start of all the text
            return cursor;
        }

        for (row_number, row) in self.row_info.iter().enumerate() {
            if coord.y <= row.bottom.as_i32() {
                cursor.row = row_number;

                let start_index = self.row_info[row_number].glyph_start;
                let end_index = self.row_info[row_number].glyph_end;
                cursor.column = end_index - start_index;
                if self.glyphs[end_index].parent != '\n' {
                    cursor.column += 1;
                }

                for i in start_index..=end_index {
                    let glyph = &self.glyphs[i];
                    if (glyph.x + glyph.width.as_f32()).as_i32() > coord.x {
                        cursor.column = i - start_index;
                        break;
                    }
                }

                return cursor;
            }
        }

        // Cursor is at the end of all the text
        cursor.row = self.row_count() - 1;
        cursor.column = self.row_info[cursor.row].glyph_end - self.row_info[cursor.row].glyph_start;
        if self.glyphs.last().unwrap().parent != '\n' {
            cursor.column += 1;
        }
        cursor
    }

    pub fn is_horizontal_overflow(&self) -> bool {
        self.actual_size.x > self.width
    }

    pub fn is_vertical_overflow(&self) -> bool {
        self.actual_size.y > self.height
    }

    pub fn prepare(&mut self, text: &str, font: &Font, draw: &TheDraw2D) {
        self.actual_size = Vec2::zero();
        self.glyphs.clear();
        self.row_info.clear();

        let mut text = text.to_owned();
        // Indicate a new line, for render and interaction only
        if text.ends_with('\n') || text.is_empty() {
            text.push('\n');
        }

        // Hack: to get the width of a normal space,
        // for that fontdue will render the tailing space with zero width
        let layout = draw.get_text_layout(font, self.font_size, "  ", LayoutSettings::default());
        let space_width = layout.glyphs().last().unwrap().x
            - layout.glyphs().first().unwrap().x
            - layout.glyphs().first().unwrap().width.as_f32();

        let layout = draw.get_text_layout(font, self.font_size, &text, LayoutSettings::default());
        let glyph_positions = layout.glyphs();
        self.glyphs = glyph_positions
            .iter()
            .map(|glyph| TheGlyph {
                parent: glyph.parent,
                start: glyph.byte_offset,
                end: glyph.byte_offset + glyph.parent.len_utf8(),
                x: glyph.x,
                width: glyph.width,
            })
            .collect();

        self.row_info = layout
            .lines()
            .unwrap()
            .iter()
            .map(|line| {
                let top = (line.baseline_y - line.max_ascent).ceil().as_usize();
                let left = self
                    .glyphs
                    .get(line.glyph_start)
                    .unwrap()
                    .x
                    .ceil()
                    .as_usize();
                let bottom = (line.baseline_y - line.min_descent).ceil().as_usize();
                let right = {
                    let last_glyph = self.glyphs.get_mut(line.glyph_end).unwrap();
                    // Manually set trailing space width
                    if last_glyph.parent == ' ' && last_glyph.width == 0 {
                        last_glyph.width = space_width.ceil().as_usize();
                    }
                    (last_glyph.x + last_glyph.width.as_f32()).ceil().as_usize()
                };

                self.actual_size.x = self.actual_size.x.max(right);
                self.actual_size.y = self.actual_size.y.max(bottom);

                TheRowInfo {
                    top,
                    left,
                    bottom,
                    right,
                    baseline: line.baseline_y.ceil().as_usize(),
                    glyph_start: line.glyph_start,
                    glyph_end: line.glyph_end,
                    highlights: None,
                }
            })
            .collect();

        if let Some(highlighter) = &self.highlighter {
            for (idx, line) in text.split('\n').enumerate() {
                if let Some(row) = self.row_info.get_mut(idx) {
                    row.highlights = Some(highlighter.highlight_line(line));
                }
            }
        }

        // Re-calculate scroll offset
        self.scroll(&Vec2::zero(), false);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_text(
        &self,
        state: &TheTextEditState,
        focused: bool,
        readonly: bool,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        font: &Font,
        draw: &TheDraw2D,
    ) {
        if let Some((start_row, end_row)) = self.visible_rows() {
            for i in start_row..=end_row {
                if focused {
                    if let Some((start, end)) = state.find_selected_range_of_row(i) {
                        self.render_selection(i, start, end, buffer, style, draw);
                    }
                }

                self.render_row(&state.rows[i], i, buffer, style, font, draw);
            }

            if focused && !readonly {
                self.render_cursor(
                    &state.cursor,
                    state.find_cursor_index(),
                    buffer,
                    style,
                    draw,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_widget(
        &self,
        shrinker: &mut TheDimShrinker,
        disabled: bool,
        embedded: bool,
        widget: &dyn TheWidget,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride = buffer.stride();
        style.draw_text_edit_border(buffer, widget, shrinker, ctx, embedded, disabled);

        if !disabled {
            ctx.draw.rect(
                buffer.pixels_mut(),
                &widget.dim().to_buffer_shrunk_utuple(shrinker),
                stride,
                &self
                    .highlighter
                    .as_ref()
                    .and_then(|h| h.background())
                    .map(|c| c.to_u8_array())
                    .unwrap_or(*style.theme().color(TextEditBackground)),
            );
        } else {
            ctx.draw.blend_rect(
                buffer.pixels_mut(),
                &widget.dim().to_buffer_shrunk_utuple(shrinker),
                stride,
                &self
                    .highlighter
                    .as_ref()
                    .and_then(|h| h.background())
                    .map(|c| c.to_u8_array())
                    .unwrap_or(*style.theme().color_disabled_t(TextEditBackground)),
            );
        }

        shrinker.shrink_by(
            self.padding.0.as_i32(),
            self.padding.1.as_i32(),
            self.padding.2.as_i32(),
            self.padding.3.as_i32(),
        );
    }

    pub fn row_baseline(&self, row_number: usize) -> usize {
        self.row_info[row_number].baseline
    }

    pub fn row_count(&self) -> usize {
        self.row_info.len()
    }

    pub fn scroll(&mut self, delta: &Vec2<i32>, visible_constrained: bool) -> bool {
        if self.row_info.is_empty() {
            self.scroll_offset = Vec2::zero();
            return true;
        }

        let previous_offset = self.scroll_offset;

        let max_width = visible_constrained
            .then(|| {
                self.visible_rows()
                    .and_then(|(start_row, end_row)| {
                        self.row_info[start_row..=end_row]
                            .iter()
                            .max_by_key(|row| row.right)
                    })
                    .map(|row| row.right)
                    .unwrap_or(self.actual_size.x)
            })
            .unwrap_or(self.actual_size.x);
        let rightmost = max_width.saturating_sub(self.width);
        self.scroll_offset.x = (self.scroll_offset.x.as_i32() + delta.x)
            .max(0)
            .as_usize()
            .min(rightmost);

        let downmost = self.actual_size.y.saturating_sub(self.height);
        self.scroll_offset.y = (self.scroll_offset.y.as_i32() + delta.y)
            .max(0)
            .as_usize()
            .min(downmost);

        previous_offset != self.scroll_offset
    }

    pub fn scroll_to_cursor(&mut self, cursor_index: usize, cursor_row: usize) {
        let text_width_before_cursor = self.get_text_left(cursor_index);

        // Check if the widget should be scrolled in order to display the cursor
        // Scroll right
        let leftmost = text_width_before_cursor;
        self.scroll_offset.x = self.scroll_offset.x.min(leftmost);
        // Scroll left
        let rightmost = (text_width_before_cursor + self.cursor_width).saturating_sub(self.width);
        self.scroll_offset.x = self.scroll_offset.x.max(rightmost);
        // Scroll down
        let upmost = self.row_info[cursor_row].top;
        self.scroll_offset.y = self.scroll_offset.y.min(upmost);
        // Scroll up
        let downmost = self.row_info[cursor_row].bottom.saturating_sub(self.height);
        self.scroll_offset.y = self.scroll_offset.y.max(downmost);
    }

    pub fn set_dim(&mut self, left: usize, top: usize, width: usize, height: usize) {
        self.left = left;
        self.top = top;
        self.width = width;
        self.height = height;
    }

    pub fn set_code_type(&mut self, code_type: &str) {
        if let Some(highlighter) = self.highlighter.as_mut() {
            highlighter.set_syntax_by_name(code_type);
        } else {
            let mut highlighter = TheCodeHighlighter::default();
            highlighter.set_syntax_by_name(code_type);
            self.highlighter = Some(Box::new(highlighter));
        }
    }

    pub fn set_code_theme(&mut self, code_theme: &str) {
        if let Some(highlighter) = self.highlighter.as_mut() {
            highlighter.set_theme(code_theme);
        } else {
            let mut highlighter = TheCodeHighlighter::default();
            highlighter.set_theme(code_theme);
            self.highlighter = Some(Box::new(highlighter));
        }
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }

    // Inclusive on both end
    pub fn visible_rows(&self) -> Option<(usize, usize)> {
        if self.row_count() == 0 {
            return None;
        }

        let start_row = self
            .row_info
            .iter()
            .enumerate()
            .find(|(_, row)| row.bottom > self.scroll_offset.y)
            .map(|(idx, _)| idx)
            .unwrap_or_default();
        let end_row = if start_row < self.row_count() - 1 {
            self.row_info[start_row + 1..]
                .iter()
                .enumerate()
                .find(|(_, row)| row.top > self.height + self.scroll_offset.y)
                .map(|(idx, _)| idx + start_row)
                .unwrap_or(self.row_count() - 1)
        } else {
            start_row
        };

        Some((start_row, end_row))
    }

    fn get_glyph_text_range(&self, index: usize) -> (usize, usize) {
        if self.glyphs.is_empty() {
            return (0, 0);
        }

        if let Some(glyph) = self.glyphs.get(index) {
            return (glyph.start, glyph.end);
        }

        let last_glyph = &self.glyphs[self.glyphs.len() - 1];
        (last_glyph.end, last_glyph.end)
    }

    fn get_text_left(&self, index: usize) -> usize {
        if self.glyphs.is_empty() {
            return 0;
        }

        if let Some(glyph) = self.glyphs.get(index) {
            return glyph.x.ceil().as_usize();
        }

        let last_glyph = &self.glyphs[self.glyphs.len() - 1];
        last_glyph.x.ceil().as_usize() + last_glyph.width
    }

    // Support single row only
    // Inclusive on both end
    // Make sure start and end are on the same row
    fn get_text_width(&self, start: usize, end: usize) -> usize {
        if self.glyphs.is_empty() {
            return 0;
        }

        let left = start.min(end);
        let right = start.max(end);
        let last_char_end = self.glyphs[right].x + self.glyphs[right].width.as_f32();
        let right_end = self
            .glyphs
            .get(right + 1)
            .map_or(last_char_end, |next_glyph| {
                if last_char_end < next_glyph.x {
                    next_glyph.x - 1.0
                } else {
                    last_char_end
                }
            });

        (right_end - self.glyphs[left].x).ceil().as_usize()
    }

    fn is_rect_out_of_visible_area(
        &self,
        left: usize,
        top: usize,
        width: usize,
        height: usize,
    ) -> bool {
        top > self.scroll_offset.y + self.height
            || top + height < self.scroll_offset.y
            || left > self.scroll_offset.x + self.width
            || left + width < self.scroll_offset.x
    }

    fn linebreak_width(&self) -> usize {
        (self.font_size * 0.5).ceil().as_usize()
    }

    fn render_cursor(
        &self,
        cursor: &TheCursor,
        cursor_index: usize,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        draw: &TheDraw2D,
    ) {
        let height = self
            .row_height()
            .saturating_sub(self.cursor_vertical_shrink * 2);

        let left = self.get_text_left(cursor_index).as_i32() - (self.cursor_width / 2).as_i32();
        let top = self.row_baseline(cursor.row).as_i32() - height.as_i32();
        if self.is_rect_out_of_visible_area(
            left.max(0).as_usize(),
            top.max(0).as_usize(),
            self.cursor_width,
            height,
        ) {
            return;
        }

        let left = (self.left.as_i32() + left - self.scroll_offset.x.as_i32())
            .max(0)
            .as_usize()
            .max(self.left);
        let top = self.top.as_i32() + top - self.scroll_offset.y.as_i32();

        let bottom = (top + height.as_i32())
            .max(0)
            .as_usize()
            .min(self.top + self.height);

        let top = top.max(0).as_usize().max(self.top);

        let stride = buffer.stride();
        let color = &self
            .highlighter
            .as_ref()
            .and_then(|hl| hl.caret())
            .map(|color| color.to_u8_array())
            .unwrap_or(*style.theme().color(TextEditCursorColor));
        draw.rect(
            buffer.pixels_mut(),
            &(left, top, self.cursor_width, bottom - top),
            stride,
            color,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_row(
        &self,
        text: &str,
        row_number: usize,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        font: &Font,
        draw: &TheDraw2D,
    ) {
        let row = &self.row_info[row_number];
        if self.is_rect_out_of_visible_area(
            row.left,
            row.top,
            row.right - row.left,
            row.bottom - row.top,
        ) {
            return;
        }

        // Find the visible text
        let glyph_start = row.glyph_start;
        let glyphs_count = row.glyph_end - glyph_start;
        let mut visible_text_start_index = 0;
        let mut visible_text_end_index = glyphs_count;
        let mut is_start_index_found = false;
        let mut chars_acc_width = 0;
        for i in 0..glyphs_count {
            if is_start_index_found && chars_acc_width >= self.scroll_offset.x + self.width {
                visible_text_end_index = i;
                break;
            }
            chars_acc_width = self.get_text_width(glyph_start, glyph_start + i);
            if !is_start_index_found && chars_acc_width >= self.scroll_offset.x {
                visible_text_start_index = i;
                is_start_index_found = true;
            }
        }

        // Render text and clip
        // Make sure row x start at 0 TODO
        let left = self.left.as_i32()
            - self.scroll_offset.x.as_i32()
            - self.get_text_left(glyph_start).as_i32();
        let top = self.top.as_i32() - self.scroll_offset.y.as_i32() + row.top.as_i32();

        let stride = buffer.stride();
        if let Some(highlights) = &row.highlights {
            let mut token_end = 0;
            for (color, token_len) in highlights {
                let token_start = token_end;
                if token_start > visible_text_end_index {
                    break;
                }
                token_end = token_start + token_len;
                if token_end < visible_text_start_index {
                    continue;
                }

                let left = left + self.get_text_left(glyph_start + token_start).as_i32();
                draw.text_rect_blend_clip(
                    buffer.pixels_mut(),
                    &vec2i(left, top - 1),
                    &(self.left, self.top, self.width, self.height),
                    stride,
                    font,
                    self.font_size,
                    &text[token_start..token_end],
                    &color.to_u8_array(),
                    TheHorizontalAlign::Center,
                    TheVerticalAlign::Center,
                );
            }
        } else {
            let left = left
                + self
                    .get_text_left(glyph_start + visible_text_start_index)
                    .as_i32();
            let row_start_index = self.get_glyph_text_range(glyph_start).0;
            let start = self
                .get_glyph_text_range(glyph_start + visible_text_start_index)
                .0
                - row_start_index;
            let end = self
                .get_glyph_text_range(glyph_start + visible_text_end_index)
                .1
                - row_start_index;
            let end = text.len().min(end);
            draw.text_rect_blend_clip(
                buffer.pixels_mut(),
                &vec2i(left, top - 1),
                &(self.left, self.top, self.width, self.height),
                stride,
                font,
                self.font_size,
                &text[start..end],
                style.theme().color(TextEditTextColor),
                TheHorizontalAlign::Center,
                TheVerticalAlign::Center,
            );
        }
    }

    fn render_selection(
        &self,
        row_number: usize,
        start: usize,
        end: usize,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        draw: &TheDraw2D,
    ) {
        let row = &self.row_info[row_number];

        let height = self.row_height() + 2 * self.selection_extend;
        let row_width = row.right - row.left;
        if self.is_rect_out_of_visible_area(row.left, row.top, row_width, self.row_height()) {
            return;
        }

        let width = self.get_text_width(start, end - 1);
        let width = if width == 0 {
            self.linebreak_width()
        } else {
            width
        };

        let left = (self.left + self.get_text_left(start)).as_i32() - self.scroll_offset.x.as_i32();
        let top = (self.top + row.baseline - height + self.selection_extend).as_i32()
            - self.scroll_offset.y.as_i32();

        let right = (left + width.as_i32())
            .max(0)
            .as_usize()
            .min(self.left + self.width);
        let bottom = (top + height.as_i32())
            .max(0)
            .as_usize()
            .min(self.top + self.height);

        let left = left.max(0).as_usize().max(self.left);
        let top = top.max(0).as_usize().max(self.top);

        let stride = buffer.stride();
        let color = &self
            .highlighter
            .as_ref()
            .and_then(|hl| hl.selection_background())
            .map(|color| color.to_u8_array())
            .unwrap_or(*style.theme().color(DefaultSelection));
        draw.blend_rect(
            buffer.pixels_mut(),
            &(left, top, right - left, bottom - top),
            stride,
            color,
        );
    }

    fn row_height(&self) -> usize {
        self.font_size.ceil().as_usize()
    }
}
