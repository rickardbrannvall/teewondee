#![allow(non_snake_case)]

use std::collections::HashMap;

pub struct AveragingContext {
    //bsk: LWEBSK,
    //sk: LWESecretKey
    xv: Vec<f64>,
    cache: HashMap<Vec<usize>,(f64, i32, f64)>,
    size: i32
}

impl AveragingContext {

    // this implementation tests calculating rolling averages for window size is N=2^k 
    // it exploites the strcture of the binary tree by chaching intermediate results
    // padding LWE ciphertexts is simulated by introducing an artificial padding parameter
    // a simulated bootstrap is only carried out when remaining padding reaches 1
    // bootstrap is optional at top level (as it may be effecient to defer)
    // it is intended to test the structure used to caclute rolling ten day T1D scores
    
    pub fn partial_mean_recursion(&mut self, iv: &[usize], p: i32, l: i32) -> (f64, i32, f64) {
        // iv is list of indices for which to find average
        // p: is padding parameter which controls how often to rescale
        // l: is level parameter for l>0 rescales all levels, otherwise manual rescale of final 
        let max_padding: i32 = 2;
        let n = iv.len();
        assert!(n>0);
        
        match self.cache.get(iv) {
            Some(&result) => {return result;}, //println!("Found in cache for: {:?}",iv); 
            _ => {}, //println!("Didnt find in cache."),
        }
        
        let m = n/2;
        if n == 1 {
            let idx = iv[0];
            //println!("n, m, p: {}, {} {}", n, m, p);
            return (1.0, p, self.xv[idx]);
        }
        assert!(n-m==m);
        let (ss, pp, rr) = self.partial_mean_recursion(&iv[..m], p, l+1);
        let (_s, _p, _r) = self.partial_mean_recursion(&iv[m..], p, l+1);
        assert!(pp==_p);
        let mut s = 0.5*ss;
        let mut rs = rr+_r;
        let mut p = pp - 1;
        if p == 1 && l > 0 { // bootstrap partial mean
            //println!("n, m, p, s: {}, {}, {}, {} - restore padding and scale", n, m, p, s);
            rs = s*rs; 
            s = 1.0; 
            p = max_padding; 
        }
        else {
            //println!("n, m, p, s: {}, {}, {}, {}", n, m, p, s);    
        }
        self.cache.insert(iv.to_vec(), (s, p, rs));
        self.size += 1;
        return (s, p, rs);
    }
    
    pub fn rolling_mean_of_2toN(&mut self, w: usize, l: i32) -> (f64, Vec<f64>) {
        // w: size of rolling window which must be 2^N
        // l: is level parameter for l>0 rescales all levels, otherwise manual rescale of final 
        let n = self.xv.len();
        let iv: Vec<usize> = (0..n).collect();
        let k = n - w + 1;
        let mut rv: Vec<f64> = vec![0.0; k];
        let base = w/2;
        let mut scale = 1.0;

        for i in 0..k {
            let m = i/base;
            let c = i - m*base;
            let mut v = (i..(i+w)).map(|i| iv[i]).collect::<Vec<_>>();
            v.rotate_right(c);
            let (s, _p, r) = self.partial_mean_recursion(&v,2,l);
            rv[i] = r;
            scale = s;
            println!("{} {} {} {:?} {}",i,m,c,v,self.size);
        }
        return (scale, rv);
    }
}

fn main() {

    //let xv: Vec<f64> = vec![179.0,175.0,183.0,189.0,183.0,185.0,188.0,182.0]; //,175.0,171.0

    let mut context = AveragingContext{
        xv: vec![179.0,175.0,183.0,189.0,183.0,185.0,188.0,182.0,175.0,171.0, 
                 179.0,175.0,183.0,189.0,183.0,185.0,188.0,182.0,175.0,171.0, 
                 179.0,175.0,183.0,189.0,183.0,185.0,188.0,182.0,175.0,171.0, 
                 179.0,175.0,183.0,189.0,183.0,185.0,188.0,182.0,175.0,171.0], //
        cache: HashMap::new(),
        size: 0
    };    
    
    //let iv: Vec<usize> = vec![0,1,2,3,4,5,6,7,8,9];
        
    /*
    println!("vec: {:?}", &context.xv);
    
    //let tst = context.xv.iter().sum::<f64>() / context.xv.len() as f64;
    //println!("tst: {}",tst);
    
    let (s, _p, r) = context.partial_mean_recursion(&vec![0,1,2,3,4,5,6,7],2);
    println!("avg1: {} ({})", r * s, context.size);

    //let (s, _p, r) = context.partial_mean_recursion(&vec![1,2,3,4,5,6,7,8],2);
    let (s, _p, r) = context.partial_mean_recursion(&vec![8,1,2,3,4,5,6,7],2);
    println!("avg2: {} ({})", r * s, context.size);

    let (s, _p, r) = context.partial_mean_recursion(&vec![8,9,2,3,4,5,6,7],2);
    println!("avg3: {} ({})", r * s, context.size);
    
    let w = 16;
    //let iv: Vec<usize> = vec![0,1,2,3,4,5,6,7];
    let n = context.xv.len();
    let iv: Vec<usize> = (0..n).collect();

    let k = iv.len() - w + 1;
    let mut av: Vec<f64> = vec![0.0; k];
    let base = w/2;
   
    for i in 0..k {
        let m = i/base;
        let l = i - m*base;
        let mut v = (i..(i+w)).map(|i| iv[i]).collect::<Vec<_>>();
        v.rotate_right(l);
        let (s, _p, r) = context.partial_mean_recursion(&v,2);
        av[i] = r * s;
        println!("{} {} {} {:?} {}",i,m,l,v,context.size);
    }
    */
    
    let av = context.rolling_mean_of_2toN(8,1);
    println!("{:?}",av)

}