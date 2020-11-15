use rand::*;
use std::io::*;

byond_fn! { cnoise_generate(precentage,smoothing_iterations, birth_limit, death_limit) {
    noise_gen(precentage, smoothing_iterations, birth_limit, death_limit).ok()
} }

fn noise_gen(prec_as_str : &str, smoothing_level_as_str : &str, birth_limit_as_str : &str, death_limit_as_str : &str)-> Result<String> {
    let prec = prec_as_str.parse::<i32>().expect("parse failed");
    let smoothing_level = smoothing_level_as_str.parse::<i32>().expect("parse failed");
    let birth_limit = birth_limit_as_str.parse::<i32>().expect("parse failed");
    let death_limit = death_limit_as_str.parse::<i32>().expect("parse failed");
    //Noise generation

    let mut zplane = vec![vec![0; 254]; 254]; // 254 but we start at 0, and since byond starts at one it is 255 byond wise.
    for i in 0..zplane.len() {
        for j in 0..zplane.len(){
            if rand::thread_rng().gen_range(0, 100) > prec {
                zplane[j][i] = 1;
            }
        }

    }

    //Smoothing part
    
    for _timer in 0..smoothing_level {
        let zplane_old = zplane.clone();
        for i in 0..zplane_old.len() {
            for j in 0..zplane_old.len(){
                let mut sum = 0;

                if j > 0{
                    sum += zplane_old[j-1][i];
                }

                if i > 0 {
                    sum += zplane_old[j][i-1];
                }

                if i > 0 && j > 0 {
                    sum += zplane_old[j-1][i-1];
                }

                if j < zplane_old.len()-1 {
                    sum += zplane_old[j+1][i];
                }

                if i < zplane_old.len()-1 {
                    sum += zplane_old[j][i+1];
                }

                if i < zplane_old.len()-1 && j < zplane_old.len()-1{
                    sum += zplane_old[j+1][i+1];
                }

                if i > 0 && j < zplane_old.len()-1 {
                    sum += zplane_old[j+1][i-1];
                }

                if j > 0 && i < zplane_old.len()-1 {
                    sum += zplane_old[j-1][i+1];
                }

                if zplane_old[j][i] == 1{
                    if sum < death_limit{
                        zplane[j][i] = 0;
                    } else{
                        zplane[j][i] = 1;
                    }
                }
                else{
                    if sum > birth_limit{
                        zplane[j][i] = 1;
                    } else{
                        zplane[j][i] = 0;
                    }
                }


            }
        }
    }


    let mut string:String = String::from("");
    for i in 0..zplane.len() {
        
        for j in 0..zplane.len(){
           if zplane[j][i] == 1 {
                string = [string, String::from("1")].join("");
           } else {
                string = [string, String::from("0")].join("");
           }
           
        }
    }   

    Ok(string)
}
