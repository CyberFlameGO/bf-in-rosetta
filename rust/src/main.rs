use std::env;
use std::fs;
use std::thread;
use std::sync::Arc;
use priomutex::spin_one::Mutex;
use regex::Regex;
use std::process;
use std::time::Instant;
use std::time;


fn main() {
    let start_time = Instant::now();
    println!("\n\nRunning\n");
    let args: Vec<String> = env::args().collect();
    let code: Vec<char> = process_bf(&args); //Gets the bf code from the file and removes comments
    let inputs: Vec<i64> = get_inputs(&args); //Gets the program inputs from command line
    // let macro_code = code;
    let macro_code = macro_scan(&code); //Condenses repeated charcters in macros (Shortcuts) 
    let braces: Vec<i32> = match_braces(&macro_code); //Matches the loops in the code
    run_bf(macro_code,braces,inputs); //Steps through processed code
    let elapsed = start_time.elapsed();
    println!("Time taken: {:.5?}", elapsed);
}


fn process_bf(args: &Vec<String>) -> Vec<char>{
    if args.len() < 2  {
        println!("How to use: \n\tmain [filename] [inputs]");
        println!("\tThere is no error checking for maxium speed.");
        println!("\tIt is recomended to test with another tool,");
        println!("\tThen blaze it with this 64 bit one");
        let error_message = format!("Args not correct, 1 filename expected, {} recived", args.len());
        throw_error(10, error_message);
    }
    let filename = &args[1];
    let file_contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    let code_pre: Vec<char> = file_contents.chars().collect();
    let code_post: Vec<char>;
    
    code_post = regex_scan(code_pre);
    

    return code_post;
}
fn macro_scan(code: &Vec<char>) -> Vec<char>{
    
    let mut code_macro: Vec<char>=vec!();
    let mut char_list: Vec<char>=vec!('0','0','0');
    let mut i: usize=0;
    
    while i < code.len()-3{
        char_list[0] = code[i];
        char_list[1] = code[i+1];
        char_list[2] = code[i+2];
        if equal_vec(&char_list){
            for x in 3..10{
                if code[i+x] != code[i] || x==255{
                    let macro_type  = match code[i]{
                        '>' => 'a',
                        '<' => 'b',
                        '+' => 'c',
                        '-' => 'd',
                         _ => 'z',
                    };
                    if macro_type == 'z'{code_macro.push(code[i]);break;}//Don't want to macro things like ",.[]"
                    code_macro.push(macro_type);
                    code_macro.push(x as u8 as char);
                    i+=x;
                    break;
                }
            }
        }
        else {
            code_macro.push(code[i]);
            
            i+=1;
        }
        
    }
    while i < code.len(){
        code_macro.push(code[i]);
        i+=1;
    }
    return code_macro;

}
fn regex_scan(code_pre: Vec<char>) -> Vec<char>{
    let regex_code = Regex::new("^[\\[\\]<>+-.,,]$").unwrap();
    let mut code_post = vec![];
    for i in 0..code_pre.len(){
        let current_char = code_pre[i];
        if regex_code.is_match(current_char.encode_utf8(&mut [4])){
            code_post.push(current_char);
        }
    }
    return code_post;
}
fn equal_vec(arr: &Vec<char>) -> bool {
    arr.iter().min() == arr.iter().max()
}

fn run_bf(code: Vec<char>,braces: Vec<i32>,inputs: Vec<i64>){
    println!("Running bf code");
    let mut memory: Vec<i64> = vec!(0);
    let mut memory_pointer: usize = 0;
    let mut code_pointer: usize = 0;
    let mut inputs_pointer: usize = 0;
    let mut outputs_temp: Vec<i64> = vec!();
    let mut caching_data = Arc::new(Mutex::new(vec!()));
    let mut caching_refrence = Arc::new(Mutex::new(vec!(0; code.len())));
    let mut handles = vec![];
    let code_arc = Arc::new(code.clone());
    println!("Varibles Initialesd");

    while code_pointer < code.len() as usize{
        if code_pointer == 104 {
            println!("\nNext Round - Caching should be ready\n")
        }
        let code_char: char = code[code_pointer];
        println!("Excuting code point {} which is {}",code_pointer,code_char);
        match code_char {
            '.' => {println!("Output: {}",memory[memory_pointer]); outputs_temp.push(memory[memory_pointer])},
            ',' => {memory[memory_pointer] = inputs[inputs_pointer]; inputs_pointer +=1; },
            '>' => memory_pointer+=1,
            '<' => {if memory_pointer != 0{memory_pointer-=1}else{throw_error(15, String::from("Bad BF code, memory pointer went below zero"))}},
            '+' => memory[memory_pointer] += 1,
            '-' => memory[memory_pointer] -= 1,
            ']' => {if memory[memory_pointer] != 0 {
                        code_pointer = braces[code_pointer] as usize;    
                    }},
            'a' => {code_pointer+=1; memory_pointer+=code[code_pointer] as usize; }, //>
            'b' => {code_pointer+=1; if memory_pointer != code[code_pointer] as usize-1{memory_pointer-=code[code_pointer] as usize;}else{throw_error(15, String::from("Bad BF code, memory pointer went below zero"))} }, //<
            'c' => {code_pointer+=1; memory[memory_pointer]+=code[code_pointer] as i64;}, //+
            'd' => {code_pointer+=1; memory[memory_pointer]-=code[code_pointer] as i64; }, //-
            '[' => {
                println!("Checking cache status");
                let mut arc_cache_status = Arc::clone(&caching_refrence);
                let mut mutex_cache_status = arc_cache_status.lock(0).unwrap();
                let mut current_cache_status = mutex_cache_status[code_pointer];
                println!("Cache Status, LoopID: {} at {}",current_cache_status,code_pointer);
                if current_cache_status == 0 {
                    mutex_cache_status[code_pointer] = -1;
                    drop(mutex_cache_status);
                    let caching_data = Arc::clone(&caching_data);
                    let code_arc = Arc::clone(&code_arc);
                    let caching_refrence = Arc::clone(&caching_refrence);
                    let mut code_pointer_local = code_pointer.clone()+1;
                    let handle = thread::spawn(move || {
                        let mut current_cache: LoopCacheMeta = LoopCacheMeta::new();
                        let mut code_arc_char = code_arc[code_pointer_local];
                        println!("Caching Thread started at {:?}, chars: {}{}{}{}{}",code_pointer_local,code_arc[code_pointer_local-3],code_arc[code_pointer_local-2],code_arc[code_pointer_local-1],code_arc[code_pointer_local],code_arc[code_pointer_local+1]);
                        let mut able_to_be_cached: bool = true;
                        let starting_position = code_pointer_local.clone();
                        let mut mutex_caching_refrence = caching_refrence.lock(5).unwrap();
                        mutex_caching_refrence[starting_position] = -1;
                        drop(mutex_caching_refrence);
                        while code_arc_char != ']' && able_to_be_cached == true{
                            
                            match code_arc_char {
                                '<' => current_cache.memory_pointer -=1,
                                '>' => current_cache.memory_pointer +=1,
                                '+' => current_cache.change_memory(1),
                                '-' => current_cache.change_memory(-1),
                                'a' => { // >
                                    code_pointer_local+=1;
                                    current_cache.memory_pointer += code_arc[code_pointer_local] as i32; 
                                }, 
                                'b' => { // <
                                    code_pointer_local+=1;
                                    current_cache.memory_pointer -= code_arc[code_pointer_local] as i32; 
                                }, 
                                'c' => { // +
                                    code_pointer_local+=1;
                                    current_cache.change_memory(code_arc[code_pointer_local] as i32);
                                },
                                'd' => { // -
                                    code_pointer_local+=1;
                                    current_cache.change_memory(-1*code_arc[code_pointer_local] as i32);
                                },
                                _   => {
                                    println!("Loop {} cannot be cached beacuse of {}",starting_position,code_arc_char);
                                    able_to_be_cached = false;
                                },
                            }
                            code_pointer_local += 1;
                            code_arc_char = code_arc[code_pointer_local];
                        }
                        current_cache.control_pointer = current_cache.memory_pointer as i32;
                        if current_cache.control_pointer != 0 {
                            println!("Loop cannot be cached due to not being static: {}",starting_position);
                            able_to_be_cached = false;
                        }
                        if able_to_be_cached == true {
                            current_cache.code_pointer = code_pointer_local as i32;
                            current_cache.loop_starting_loc = starting_position as i32;
                            let mut mutex_caching_data = caching_data.lock(4).unwrap();
                            let mut mutex_caching_refrence = caching_refrence.lock(4).unwrap();
                            mutex_caching_data.push(current_cache);
                            mutex_caching_refrence[starting_position-1] = mutex_caching_data.len() as i32; //One more than actual index
                            println!("Cache {} compleated sucesfully loop num {:?}",starting_position,mutex_caching_data.len()-1);
                            drop(mutex_caching_data);
                            drop(caching_data);
                            drop(mutex_caching_refrence);
                        }
                        else { // Error caused cache to not compleate
                            drop(caching_data);
                        }

                    });
                    handles.push(handle);
                } else if current_cache_status > 0{
                    println!("Wowe, a cached loop.");
                    let mutex_cache = Arc::clone(&caching_data);
                    println!("Waiting for lock");
                    let unlocked_cache = mutex_cache.lock(0).unwrap();
                    let cache = unlocked_cache[current_cache_status as usize-1].clone();
                    println!("Got lock");
                    drop(mutex_cache_status);
                    drop(unlocked_cache);
                    let mut i: usize = 0;
                    let control_memory =  memory[memory_pointer+cache.control_pointer as usize];
                    println!("Ready to excute instructions");
                    println!("Instructions: {:#?}",cache);
                    println!("Current Memory pointer: {}, with control: {}",memory_pointer,control_memory);
                    while i < cache.instructions.len() {
                        print!("Excuting instruction {} that adds {}*{} to memory pointer {}, before: {}",i,cache.instructions[i][1], control_memory,memory_pointer+cache.instructions[i][0] as usize, memory[memory_pointer+cache.instructions[i][0] as usize]);
                        memory[memory_pointer+cache.instructions[i][0] as usize] += cache.instructions[i][1] as i64 * control_memory;
                        println!(", after: {}",memory[memory_pointer+cache.instructions[i][0] as usize]);
                        // if cache.instructions[i][1].is_negative() {
                        //     println!("Instruction is negitive");
                        //     memory[memory_pointer+cache.instructions[i][0] as usize] -= cache.instructions[i][1].wrapping_abs() as i64 * memory[cache.control_pointer as usize];
                        // } else {
                        //     println!("Instruction is postive");
                        //     memory[memory_pointer+cache.instructions[i][0] as usize] = memory[memory_pointer+cache.instructions[i][0] as usize] + cache.instructions[i][1] as i64 * memory[cache.control_pointer as usize];
                        // }
                        i+=1;
                    }
                    memory[memory_pointer+cache.control_pointer as usize] = 0;
                    memory_pointer = add_to_usize(memory_pointer, cache.memory_pointer);
                    code_pointer = cache.code_pointer as usize;

                }
                else {
                    drop(mutex_cache_status);
                }
            }
            _ => (),
        }
        println!("memory (Pointer at {}):\n{:?}\n\n",memory_pointer,memory);
        code_pointer+=1;
        while memory_pointer >= memory.len()-1{
            memory.push(0);
        }
    }
    println!("BF excution done");
    for handle in handles {
        handle.join().unwrap();
    }
    println!("\nOutputs\n{:?}\n",outputs_temp)
    //let mut caching_dataLocal = caching_data.lock(7).unwrap();
    //println!("Carching refrence: {:?}",Arc::clone(&caching_refrence).lock(7).unwrap());
}
fn get_inputs(args: &Vec<String>)-> Vec<i64>{
    let mut inputs: Vec<i64> = vec![];
    for i in 2..args.len(){
        if !args[i].parse::<i64>().is_err(){
            inputs.push(args[i].parse::<i64>().unwrap());
        }
        else{
            throw_error(5,String::from(format!("Input {} is not a number (i64)",i-1)));
        }
        
	}
    println!("Inputs: {:?}",inputs);
    return inputs;
}
fn match_braces(code_post: &Vec<char>)-> Vec<i32>{
    let mut nested_level: i32 = 1;
    let mut bracket_left: Vec<Vec<i32>> = vec!();
    let mut bracket_right: Vec<i32> = vec!();
    for i in 0..code_post.len(){
        if code_post[i] == '['{
            nested_level += 1;
            bracket_left.push(vec!(nested_level,i as i32));
            bracket_right.push(0);
        }
        else if code_post[i] == ']'{
            let mut x: usize =  bracket_left.len() -1;
            #[allow(unused_comparisons)]
            'scan_for_match: while x >= 0 {
                if  bracket_left[x][0] == nested_level{
                    bracket_right.push(bracket_left[x][1]);
                    break 'scan_for_match;
				}
                x -= 1;
			}
            nested_level -= 1;
        }
        else{
            bracket_right.push(-2);
        }
    }
    return bracket_right;
}


fn throw_error(error_code: i32,message: std::string::String){
    println!("The program encounted a error:");
    println!("Code: {}, Message: {}",error_code,message);
    process::exit(error_code);
}

fn add_to_usize(usizeNum: usize, i32Num: i32) -> usize{
    if i32Num.is_negative() {
        return usizeNum - i32Num.wrapping_abs() as usize;
    } else {
        return usizeNum + i32Num as usize;
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct LoopCacheMeta {
    instructions: Vec<Vec<i32>>,
    control_pointer: i32,
    code_pointer: i32,
    memory_pointer: i32,
    loop_starting_loc: i32,
}
impl LoopCacheMeta {
    pub fn change_memory(&mut self, amount: i32) {
        let mut instruction: Vec<i32> = vec!();
        instruction.push(self.memory_pointer);
        instruction.push(amount.clone());
        self.instructions.push(instruction);
        return;
    }
    pub fn new() -> LoopCacheMeta{
        return LoopCacheMeta {
            instructions: vec!(),
            code_pointer: 0,
            control_pointer: 0,
            memory_pointer: 0,
            loop_starting_loc: 0
        }
    }
}