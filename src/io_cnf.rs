use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
pub fn read_cnf_file(filename: &str) -> (i32, i32, Vec<Vec<i32>>) {
    let mut nr_lit = -1;
    let mut nr_cl = -1;
    let path = Path::new(&filename);
    let data = File::open(&path).expect("file not found");
    let mut b_reader = BufReader::new(data);
    let mut clauses: Vec<Vec<i32>> = Vec::new();
    let mut temp: Vec<i32> = Vec::new();
    let mut new = false;
    for line in b_reader.lines() {
        let l = line.unwrap();
        if l.starts_with("c") {
            continue;
        } else if l.starts_with("p") {
            let t: Vec<&str> = l.split(" ").collect();
            nr_lit = t[2].to_owned().parse().unwrap();
            nr_cl = t[3].to_owned().parse().unwrap();
        } else {
            let t: Vec<&str> = l.split(" ").collect();
            let mut i: Vec<i32> = t.into_iter().map(|x| x.parse().unwrap()).collect();
            i = i
                .into_iter()
                .filter(|&x| {
                    if x == 0 {
                        new = true
                    }
                    x != 0
                })
                .collect();
            temp.append(&mut i);
            if new {
                clauses.push(temp);
                temp = vec![];
                new = false;
            }
        }
    }
    (nr_lit, nr_cl, clauses)
}

#[test]
fn test_1() {
    let (nr_lit, nr_cl, clauses) = read_cnf_file("test.txt");
    assert_eq!(nr_lit, 9);
    assert_eq!(nr_cl, 12);
    assert_eq!(nr_cl, clauses.len() as i32);
}
