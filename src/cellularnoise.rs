use rand::*;
use crate::error::Result;
use std::fmt::Write;

byond_fn! { cnoise_generate(precentage,smoothing_iterations, birth_limit, death_limit) {
    noise_gen(precentage, smoothing_iterations, birth_limit, death_limit).ok()
} }

fn noise_gen(prec_as_str : &str, smoothing_level_as_str : &str, birth_limit_as_str : &str, death_limit_as_str : &str)-> Result<String> {
    let prec = prec_as_str.parse::<i32>()?;
    let smoothing_level = smoothing_level_as_str.parse::<i32>()?;
    let birth_limit = birth_limit_as_str.parse::<i32>()?;
    let death_limit = death_limit_as_str.parse::<i32>()?;
    //Noise generation

    let mut zplane = vec![vec![false; 254]; 254]; // 254 but we start at 0, and since byond starts at one it is 255 byond wise.
    for i in 0..zplane.len() {
        for j in 0..zplane[i].len(){
            if rand::thread_rng().gen_range(0, 100) > prec {
                zplane[i][j] = true;
            }
        }

    }

    //Smoothing part
    
    for _timer in 0..smoothing_level {
        let zplane_old = zplane.clone();
        for i in 0..zplane_old.len() {
            for j in 0..zplane_old[i].len(){
                let mut sum = 0;

                if j > 0{
                    sum += if zplane_old[i-1][j] {1} else {0};
                }

                if i > 0 {
                    sum += if zplane_old[i][j-1]  {1} else {0};
                }

                if i > 0 && j > 0 {
                    sum += if zplane_old[i-1][j-1] {1} else {0};
                }

                if j < zplane_old[i].len()-1 {
                    sum += if zplane_old[i+1][j] {1} else {0};
                }

                if i < zplane_old.len()-1 {
                    sum += if zplane_old[i][j+1] {1} else {0};
                }

                if i < zplane_old.len()-1 && j < zplane_old.len()-1{
                    sum += if zplane_old[i+1][j+1] {1} else {0};
                }

                if i > 0 && j < zplane_old[i].len()-1 {
                    sum += if zplane_old[i+1][j-1] {1} else {0};
                }

                if j > 0 && i < zplane_old.len()-1 {
                    sum += if zplane_old[i-1][j+1] {1} else {0};
                }

                if zplane_old[i][j] == true{
                    if sum < death_limit{
                        zplane[i][j] = false;
                    } else{
                        zplane[i][j] = true;
                    }
                }
                else{
                    if sum > birth_limit{
                        zplane[i][j] = true;
                    } else{
                        zplane[i][j] = false;
                    }
                }


            }
        }
    }

    let mut string:String = String::from("");
    for i in 0..zplane.len() {
        
        for j in 0..zplane[i].len(){
           if zplane[i][j] == true {
                write!(string,"1");
           } else {
                write!(string,"0");
           }
           
        }
    }   

    Ok(string)
}
