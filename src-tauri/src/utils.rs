use rand::Rng;

pub fn generate_random_name() -> String {
    let adjs = ["Fast", "Swift", "Quiet", "Happy", "Brave", "Cool", "Lazy"];
    let animals = ["Crab", "Panda", "Tiger", "Fox", "Whale", "Eagle", "Cat"];

    let mut rng = rand::thread_rng();
    let adj = adjs[rng.gen_range(0..adjs.len())];
    let animal = animals[rng.gen_range(0..animals.len())];
    let num: u32 = rng.gen_range(100..999);

    format!("{}-{}-{}", adj, animal, num)
}
