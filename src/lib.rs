mod hanja_char;
mod hanja_word;
mod dueum;

use std::{collections::HashMap, error::Error};

const KO_START:u32 = 44032;
const KO_END:u32 = 55203;

const CHI_S1:u32 = 13312;
const CHI_E1:u32 = 19903;

const CHI_S2:u32 = 19968;
const CHI_E2:u32 = 40959;

const CHI_S3:u32 = 63744;
const CHI_E3:u32 = 64045;

const CHI_S4:u32 = 64048;
const CHI_E4:u32 = 64109;


pub fn load_dictionary() 
        -> Result<(HashMap<char, char>, HashMap<char, char>, HashMap<String, String>), Box<dyn Error>> {
    
    //1. 기본한자 변환 사전
    let char_dic = hanja_char::HANJA_BASIC.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                let key_char = parts[0].trim().chars().next().unwrap();
                let val_char = parts[1].trim().chars().next().unwrap();
                Some((key_char, val_char))
            } else {
                None
            }
        })
        .collect::<HashMap<char, char>>();


    //2. 두음법칙 사전
    //dueum::DUEUM은 ("냥,양\n") 형태의 여러 라인으로 구성되어 있다. 
    //모든 라인을 읽고, 각 라인 별로 콤마를 기준으로 split하여, 앞 문자와 뒤 문자를 각각 key와 value로 설정한다.
    //이때, key와 value는 모두 char로 변환하여 저장한다.
    let dueum_dic = dueum::DUEUM.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                let key_char = parts[0].trim().chars().next().unwrap();
                let val_char = parts[1].trim().chars().next().unwrap();
                Some((key_char, val_char))
            } else {
                None
            }
        })
        .collect::<HashMap<char, char>>();

    //3. 불규칙 변환 한자사전
    // hanja_word::HANJA_SPECIAL은 ("女子,여자\n") 형태의 여러 라인으로 구성되어 있다.
    //모든 라인을 읽고, 각 라인 별로 콤마를 기준으로 split하여, 앞 문자와 뒤 문자를 각각 key와 value로 설정한다.
    //이때, key와 value는 모두 String으로 변환하여 저장한다.
    let word_dic = hanja_word::HANJA_SPECIAL.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                let key_str = parts[0].trim().to_string();
                let val_str = parts[1].trim().to_string();
                Some((key_str, val_str))
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>();  

    Ok((char_dic, dueum_dic, word_dic))
}


pub async fn convert_str(
    input_str:&str,         
    char_dic:&HashMap<char,char>, 
    dueum_dic:&HashMap<char,char>,
    word_dic:&HashMap<String, String>) -> Option<String>{

    //1. obtain char array from input_str
    let mut c_iter = input_str.chars().peekable();           

    // 2. convert to hangul 
    let mut buf:String = String::new(); 
    let mut is_exist_chi:bool = false;     
    loop {    
        //2.1 pick a word only contains chinese character
        let mut word:String = String::new();   
        let mut tmp_iter = c_iter.clone();
        while let Some(c) = tmp_iter.peek() {
            if is_chi(&c) {word.push(*c); tmp_iter.next();}
            else {break;}
        }

        //2.2 if 'word' is not empty, check whether it is in the word_dic or not.
        //    if exist, append the value to w_buf and continue.
        //    if not, revert the c_iter and continue.
        if word.len() > 0 {
            match word_dic.get(&word) {
                Some(val) => {
                    buf.push_str(val); 
                    is_exist_chi = true; 
                    c_iter = tmp_iter; // Move the main iterator forward
                    continue;
                },
                None => {}
            }
        }
        
        //2.3 pick a char. if c is None, it's end of file
        let c = match c_iter.next() { 
            Some(ch) => {ch},  None => {break;} 
        };
        let mut new_c = c.clone();

        //2.4 if hanja then convert to hangul else not change       
        if is_chi(&c) {         
            match char_dic.get(&c) {
                Some(val) => {new_c = *val; is_exist_chi = true; },  
                None => {},
            };          

            //2.5. dueum law(두음법칙)
            if let Some(c_peek) = c_iter.peek(){                
                if is_kor_or_chi(&c_peek) { // if next char is exist
                    match dueum_dic.get(&new_c) {
                        Some(ch) => {new_c = *ch;},
                        None => {},
                    }
                }                     
            }          
        }
        buf.push(new_c);
    }
    
    //  if there is no chinese character in the string, return None.
    //   if exist, return the converted string.
    if !is_exist_chi {return None;}
    Some(buf)        
}    

// whether c is chinese character or not
fn is_chi(c:&char) -> bool {
    let n = *c as u32;
    if  (n >= CHI_S1 && n <= CHI_E1) || (n >= CHI_S2 && n <= CHI_E2) || 
        (n >= CHI_S3 && n <= CHI_E3) || (n >= CHI_S4 && n <= CHI_E4) { 
        true 
    }else { false }  
}

// whether c is (korean or chinese character) or not
fn is_kor_or_chi(c:&char) -> bool {
    let n = *c as u32;
    if (n >= KO_START && n <= KO_END) || 
       ( (n >= CHI_S1 && n <= CHI_E1) || (n >= CHI_S2 && n <= CHI_E2) || 
         (n >= CHI_S3 && n <= CHI_E3) || (n >= CHI_S4 && n <= CHI_E4)) {
        true
    }else {
        false
    }    
}


