extern crate rand;

use rand::thread_rng;
use std::fs::File;

byond_fn! { cnoise_generate(precentage,smoothing_iterations,name) {
    noise_gen(precentage, smoothing_iterations, name)
} }

byond_fn! { cnoise_get_at_coordinates(name,xcord,ycord) {
    get_tile_value_from_file(name,xcord,ycord)
} }

fn noise_gen(prec : i32, smoothing_level : i32, name : String) {

    //Noise generation

    let mut zplane = vec![vec![0; 255]; 255];
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


                if sum > 4{
                    zplane[j][i] = 1;
                } else{
                    zplane[j][i] = 0;
                }


            }
        }
    }


    let mut file = _make_file(name).expect("create failed");


    for i in 0..zplane.len() {
        let mut string:String = String::from("");
        for j in 0..zplane.len(){
           if zplane[j][i] == 1 {
                string = [string, String::from("1")].join("");
           } else {
                string = [string, String::from("0")].join("");
           }
           
        }
        string = [string, String::from("\n")].join("");
        file.write_all(string.as_bytes()).expect("write failed");
    }   

    return true
}

fn _make_file(name : String) -> std::io::Result<File> {
    let f:File = File::create(name)?;
    Ok(f)
}

fn get_tile_value_from_file(name : String, xcord : i32, ycord : i32) -> String{
    let f:File = _open_file(name).expect("create failed");
    // uses a reader buffer
    let  reader = BufReader::new(f);
    let mut x_local_cord = 0;
    let mut y_local_cord = 0;
    for line in reader.lines(){
        for character in line.expect("lines failed").chars(){
            if x_local_cord == xcord && y_local_cord == ycord {
                return character.to_string();
            }
            x_local_cord += 1;
        }
        y_local_cord += 1;
    }

    return "-1".to_string()

}

fn _open_file(name : String) -> std::io::Result<File> {
    let f:File = File::open(name)?;
    Ok(f)
}
