/// String similarity check (Levenshtein distance)
pub fn is_similar(a: &str, b: &str) -> bool {
    let mut distance: f64 = 0.0;

    let max_len = a.len().max(b.len());
    let min_len = a.len().min(b.len());

    let len_diff = max_len - min_len;

    // letfejaifjwoiafjewiao !~ let
    // letkj ~ let
    if len_diff > 5 {
        return false;
    }

    let mut a_chars = a.chars();
    let mut b_chars = b.chars();

    loop {
        let Some(current_a) = a_chars.next() else {
            break;
        };
        let Some(current_b) = b_chars.next() else {
            break;
        };

        if current_a != current_b {
            distance += 1.0;
        }
    }

    let p = 1.0 - (distance / (max_len as f64));

    p >= 0.70
}
