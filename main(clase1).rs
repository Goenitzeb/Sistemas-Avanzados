fn main() {


    //let x= 5;
    //x=10; esto no es valido;

    //let mut y =10;
    //y =20;
    let x = 5;
    let x = x+1;
    println!("El valor de x es:{}", x);

    ///variables ///
    let entero: i32 = 42;
    let flotante: f64 = 3.1416;
    let booleano: bool = true;
    let caracter: char = 'a';
//tupla -> struct //creacion de tupla llamada firulais
    let firulais: (i32, f64, char)=(43, 3.1416, 'b');
    let arreglo: [i32; 3] = [1,2,3];
    println!("Tupla(firulais) forma1: {:?}", firulais);

    println!("Tupla (firulais) forma2: ({},{},{})", firulais.0, firulais.1, firulais.2);
}
