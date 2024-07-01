pub fn _code_box_scale(start: isize, end: isize, arrow: f64, scale_len: usize) -> String {
    // Step 1: Calculate positions within the scale
    let start_pos = 0;
    let end_pos = scale_len - 1;
    let arrow_pos = (((arrow - start as f64) as f64 / (end - start) as f64 * (scale_len - 1) as f64) - (arrow.to_string().len() as f64 / 2.)).round() as usize;
    
    // Step 2: Create the scale
    let mut scale = vec!['-'; scale_len];
    scale[start_pos] = '|';
    scale[end_pos] = '|';

    let mut bottom_scale = vec![' '; scale_len];
    bottom_scale[arrow_pos] = '^';
    
    // Step 3: Create the markers string
    let mut markers = vec![' '; scale_len];
    let start_marker = start.to_string();
    let end_marker = end.to_string();
    let arrow_marker = arrow.to_string();

    for (i, ch) in start_marker.chars().enumerate() {
        markers[start_pos + i] = ch;
    }
    for (i, ch) in end_marker.chars().enumerate() {
        markers[end_pos - end_marker.len() + 1 + i] = ch;
    }
    let mut bottom_markers = vec![' '; scale_len];
    for (i, ch) in arrow_marker.chars().enumerate() {
        bottom_markers[arrow_pos + i] = ch;
    }

    // Combine the markers and scale into the final result
    format!("```\n{}\n{}\n{}\n{}\n```", markers.iter().collect::<String>(), scale.iter().collect::<String>(), bottom_scale.iter().collect::<String>(), bottom_markers.iter().collect::<String>())
}