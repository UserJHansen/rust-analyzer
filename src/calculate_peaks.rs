// Returns a filtered vector of peaks in the given vector
pub fn calculate_peaks(arr: Vec<usize>) -> (f64, Vec<usize>) {
    let avg = arr.iter().filter(|v| **v != 0).sum::<usize>() / arr.len() + arr.len() / 4;

    let tempresults = arr.iter().enumerate().filter(|(_, v)| **v > avg);

    if tempresults.clone().count() == 1 {
        return (
            f64::from(tempresults.clone().last().unwrap().0 as i32) / arr.len() as f64,
            tempresults.map(|(i, _)| i).collect(),
        );
    } else {
        let mut results = Vec::new();
        let mut last = 0;
        let mut running_total = 0f64;

        for (i, v) in tempresults {
            if i - last > 1 {
                results.push(i);
                running_total += f64::from(*v as i32);
            }
            last = i;
        }

        return (running_total / arr.len() as f64, results);
    }
}
