
fn rec_fun<F>(cb: &mut F, count: u64) where F: FnMut(u64) {
    cb(count);
    if count == 0 {
        return;
    }
    rec_fun(cb, count-1);
}


fn main() {

    let mut nums = vec![];

    rec_fun(&mut |c| nums.push(c), 10);

    for i in &nums {
        println!("i: {}",i);
    }

}
