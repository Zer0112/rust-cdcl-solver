mod io_cnf;
use std::collections::HashSet;

use io_cnf::read_cnf_file;

#[derive(Debug)]
struct Watcher {
    nr_lit: usize,
    nr_cl: usize,
    clauses: Vec<Vec<i32>>,

    deep: usize,
    trail: Vec<i32>,
    trail_lvl: Vec<usize>,
    free_lit: HashSet<i32>,

    links_literals: Vec<Vec<Vec<i32>>>,

    watched_literals: Vec<bool>,

    conflict: bool,
}

impl Watcher {
    fn new(nr_lit: usize, nr_cl: usize, clauses: Vec<Vec<i32>>) -> Watcher {
        let mut free_lit = HashSet::new();
        for i in 1..(nr_lit + 1) {
            free_lit.insert(i as i32);
        }
        let mut watcher = Watcher {
            nr_lit: nr_lit,
            nr_cl: nr_cl,
            clauses: clauses,
            deep: 0,
            trail: vec![],
            trail_lvl: vec![],
            free_lit: free_lit,
            links_literals: vec![vec![]],
            watched_literals: vec![false; 2 * nr_lit + 2],
            conflict: false,
        };
        watcher.generate_watched_literals();
        return watcher;
    }

    fn generate_watched_literals(&mut self) {
        // let mut watched_literals = vec![false; self.nr_lit];
        let mut unit_queue = vec![];
        for cl in self.clauses.iter() {
            let mut nr_w = 0;
            let mut l = 0;
            for lit in cl.iter() {
                let i = Watcher::get_index_lit(*lit);
                self.watched_literals[i] = true;
                nr_w += 1;
                if nr_w == 2 {
                    break;
                }
                if nr_w == 1 {
                    l = *lit;
                }
            }
            if nr_w == 0 {
                self.conflict = true
            } else if nr_w == 1 {
                unit_queue.push(l);
            }
        }
        for i in unit_queue.into_iter() {
            self.unit_prop(i);
        }
    }
    fn generate_watched_literals_one_clause(&mut self) {
        todo!();
    }
    fn get_index_lit(lit: i32) -> usize {
        // all positive literals are stored at 2xlit
        // all negative literals are stored at 2x|lit|+1
        if lit > 0 {
            return 2 * lit as usize;
        } else {
            return (2 * (-lit) + 1) as usize;
        }
    }

    fn has_free_literals(&mut self) -> bool {
        let finished = self.trail.len() == self.nr_cl as usize;
        return !finished;
    }

    fn get_literal(&mut self) -> i32 {
        let lit = self.free_lit.iter().next().unwrap();
        return *lit;
    }

    fn set_literal(&mut self, lit: i32) {
        self.free_lit.remove(&lit);
        self.trail.push(lit);
        self.trail_lvl.push(self.deep);
        println!("{:?}", self.trail);
        if self.is_watched(-lit) {
            let mut keep_it_watched = false;
            // TODO: not sure if right
            for other_watched_list in self.find_replacement_watched(-lit).iter() {
                let (found, other_w, other_w_sat) =
                    self.do_replace_watched(other_watched_list, lit);
                if !found {
                    match other_w_sat {
                        Some(true) => keep_it_watched = true,
                        Some(false) => {
                            self.conflict = true;
                            break;
                        }
                        None => self.unit_prop(other_w.unwrap()),
                    }
                }
                if keep_it_watched {
                    // rare case of needing to keep it watched in one of the watched clauses but not in the others
                    let index = Watcher::get_index_lit(lit);
                    self.watched_literals[index] = true;
                }
            }
        }
    }

    fn unit_prop(&mut self, lit: i32) {
        self.set_literal(lit);
    }

    fn do_replace_watched(
        &mut self,
        other_watch_list: &Vec<i32>,
        lit: i32,
    ) -> (bool, Option<i32>, Option<bool>) {
        let mut other_w: Option<i32> = None;
        let mut other_w_sat: Option<bool> = None;
        let mut found_watched = false;
        for w_canidate in other_watch_list.iter() {
            if self.is_watched(*w_canidate) {
                other_w = Some(*w_canidate);
                if !self.free_lit.contains(&w_canidate.abs()) {
                    if self.trail.contains(&w_canidate) {
                        found_watched = true;
                        other_w_sat = Some(true);
                    } else {
                        other_w_sat = Some(false);
                    }
                    break;
                }
            } else if self.free_lit.contains(&w_canidate.abs()) {
                found_watched = true;
                self.replace_watched(-lit, *w_canidate);
                break;
            } else {
                let s = self.trail.contains(w_canidate);
                if s {
                    found_watched = true;
                    self.replace_watched(-lit, *w_canidate);
                }
            }
        }
        return (found_watched, other_w, other_w_sat);
    }

    fn find_replacement_watched(&mut self, lit: i32) -> Vec<Vec<i32>> {
        // rust being rust - need to find something better than clone
        todo!();
        let index = Watcher::get_index_lit(lit);
        let lst = self.links_literals[index].clone();
        return lst;
    }

    fn replace_watched(&mut self, unwatched: i32, watched: i32) {
        let i_unwatched = Watcher::get_index_lit(unwatched);
        let i_watched = Watcher::get_index_lit(watched);
        self.watched_literals[i_unwatched] = false;
        self.watched_literals[i_watched] = true;
    }

    fn find_conflict_level(&mut self) -> usize {
        todo!()
    }

    fn backtrack(&mut self, lvl: usize) {
        while self.trail_lvl.last().unwrap() == &lvl {
            self.trail_lvl.pop();
            let l = self.trail.pop().unwrap();
            self.free_lit.insert(l);
        }
    }

    fn is_watched(&mut self, lit: i32) -> bool {
        let index = Watcher::get_index_lit(lit);
        let w = self.watched_literals[index];
        return w;
    }
}

#[derive(Debug)]
struct Solver {
    sat: Option<bool>,
    watcher: Watcher,
    model: Vec<i32>,
}

impl Solver {
    fn new(file: &str) -> Solver {
        let (nr_lit, nr_cl, clauses) = read_cnf_file("test.txt");
        let watcher = Watcher::new(nr_lit, nr_cl, clauses);
        Solver {
            sat: None,
            watcher: watcher,
            model: Vec::new(),
        }
    }

    fn solve(&mut self) -> bool {
        match self.sat {
            Some(sat) => sat,
            _ => {
                self.sat = Some(self.cdcl());
                self.solve()
            }
        }
    }

    fn cdcl(&mut self) -> bool {
        if self.watcher.conflict {
            return false;
        }
        while self.watcher.has_free_literals() {
            self.watcher.deep += 1;
            let pick_lit = self.watcher.get_literal();
            self.watcher.set_literal(pick_lit);
            if self.watcher.conflict {
                let b_lvl = self.watcher.find_conflict_level();
                if b_lvl == 0 {
                    return false;
                }
                self.watcher.backtrack(b_lvl);
            }
        }
        self.model = self.watcher.trail.clone();
        return true;
    }
}

fn main() {
    println!("Hello, world!");
    println!("{:?}", read_cnf_file("test.txt"));
    let mut s = Solver::new("test.txt");
    print!("{:?}", s.solve());
    println!("{:?}", s);
}
