/// xvi files are used in Therion for background vector (drawing).
/// Usulally Imports from Pocket Topo etc are converted to xvi
/// by xtherion automatically. Code for generating them
/// is found in xtherion/me_import.tcl.
///
/// The format is undocumented but relatively simple.
/// It can contain Stations, Shots ("Laser lines", path of the graph) and drawing polylines ("Sketchlines") with a specific color.
///
///
/// See https://github.com/marcocorvi/topodroid/issues/36
/// https://www.mail-archive.com/therion@speleo.sk/msg07359.html
///
/// /// Code for writing xvi:
/// * https://github.com/therion/therion/blob/v6.0.6/thexpmap.cxx#L386
/// * https://github.com/therion/therion/blob/v6.0.6/xtherion/me_import.tcl#L38
///
/// Code for displaying XVI:
/// * https://github.com/therion/therion/blob/v6.0.6/xtherion/me_imgs.tcl#L340
/// * https://github.com/therion/therion/blob/v6.0.6/xtherion/me_imgs.tcl#L455
///
///
///
/// The only really obscure data is XVIgrid. XVIgrid has 8 Values, e.g.
///
/// ```text
/// set XVIgrid {738.9763779527558 -109.84251968503936 39.370078740157474 0.0 0.0 39.370078740157474 84 93}
/// ```
///
/// This seems to be:
/// * minx (origin / bottom left x)
/// * miny (origin / bottom left y)
/// * grid size?
/// * 0.0
/// * 0.0
/// * grid size?
/// * number of grid elements in x direction
/// * number of grid elements in y direction
///

pub mod xvi {
    use crate::pest::*;
    use pest::iterators::Pair;
    use vek::geom::repr_c::Aabr;
    use vek::Vec2;
    use std::fs;

    #[derive(Parser)]
    #[grammar = "xvi.pest"]
    pub struct XVIParser;

    #[derive(Debug)]
    pub struct Station {
        pub pos: Vec2<f32>,
        pub name: String,
    }

    #[derive(Debug)]
    pub struct Shot {
        pub start: Vec2<f32>,
        pub end: Vec2<f32>,
    }

    #[derive(Debug)]
    pub struct Sketchline {
        pub color: String,
        pub points: Vec<Vec2<f32>>,
    }

    #[derive(Debug)]
    pub struct Xvi {
        stations: Vec<Station>,
        shots: Vec<Shot>,
        sketchlines: Vec<Sketchline>,
        bounds: Aabr<f32>,
    }

    pub fn parse_file(filename: String) -> Xvi {
        parse_string(fs::read_to_string(filename).expect("cannot read file"))
    }

    pub fn parse_string(unparsed_file: String) -> Xvi {
        let file = XVIParser::parse(Rule::file, &unparsed_file)
            .expect("unsuccessful parse") // unwrap the parse result
            .next()
            .unwrap(); // get and unwrap the `file` rule; never fails

        fn parse_coordinates<R>(pair: Pair<R>) -> Vec2<f32>
        where
            R: RuleType,
        {
            let mut inner_rules = pair.into_inner();
            {
                Vec2 {
                    x: inner_rules.next().unwrap().as_str().parse().unwrap(),
                    y: inner_rules.next().unwrap().as_str().parse().unwrap(),
                }
            }
        }

        let mut stations: Vec<Station> = Vec::new();
        let mut shots: Vec<Shot> = Vec::new();
        let mut sketchlines: Vec<Sketchline> = Vec::new();
        let mut boundlist: Vec<Vec2<f32>> = Vec::new();

        for line in file.into_inner() {
            match line.as_rule() {
                Rule::xvigrids => {
                    let mut inner_rules = line.into_inner(); // { name }
                    let gridsval = inner_rules.next().unwrap().as_str();
                    let gridsunit = inner_rules.next();
                    println!("xvigrids {:#?} {:#?}", gridsval, gridsunit);
                }
                Rule::xvigrid => {
                    let mut inner_rules = line.into_inner();
                    let origin = parse_coordinates(inner_rules.next().unwrap());
                    let gridsize1: f32 = inner_rules.next().unwrap().as_str().parse().unwrap();
                    let unknown1: f32 = inner_rules.next().unwrap().as_str().parse().unwrap();
                    let unknown2: f32 = inner_rules.next().unwrap().as_str().parse().unwrap();
                    let gridsize2: f32 = inner_rules.next().unwrap().as_str().parse().unwrap();
                    let gridextendx: i32 = inner_rules.next().unwrap().as_str().parse().unwrap();
                    let gridextendy: i32 = inner_rules.next().unwrap().as_str().parse().unwrap();
                    println!(
                        "{:?} {:#?} {:#?}  {:#?} {:#?}",
                        origin, gridsize1, gridsize2, gridextendx, gridextendy
                    );
                }
                Rule::xvistations => {
                    for station in line.into_inner() {
                        let mut inner_rules = station.into_inner();
                        let station = {
                            Station {
                                pos: parse_coordinates(inner_rules.next().unwrap()),
                                name: inner_rules.next().unwrap().to_string(),
                            }
                        };
                        boundlist.push(station.pos);
                        stations.push(station);
                    }
                }
                Rule::xvishots => {
                    for shot in line.into_inner() {
                        let mut inner_rules = shot.into_inner();
                        let shot = {
                            Shot {
                                start: parse_coordinates(inner_rules.next().unwrap()),
                                end: parse_coordinates(inner_rules.next().unwrap()),
                            }
                        };
                        boundlist.push(shot.start);
                        boundlist.push(shot.end);
                        shots.push(shot);
                    }
                }
                Rule::xvisketchlines => {
                    for shot in line.into_inner() {
                        let mut inner_rules = shot.into_inner();
                        let color = inner_rules.next().unwrap().as_str().to_string(); // .parse().unwrap();
                        let mut points: Vec<Vec2<f32>> = Vec::new();
                        while let Some(coords) = inner_rules.next() {
                            let coords = parse_coordinates(coords);
                            boundlist.push(coords);
                            points.push(coords)
                        }
                        sketchlines.push({
                            Sketchline {
                                color: color,
                                points: points,
                            }
                        });
                    }
                }
                Rule::EOI => (),
                _ => unreachable!("no handler for {:#?}", line.as_rule()),
            }
        }

        let mut biter = boundlist.into_iter();
        let mut bounds: Aabr<f32> = Aabr::new_empty(biter.next().unwrap());
        while let Some(coords) = biter.next() {
            bounds.expand_to_contain_point(coords);
        }
        bounds.make_valid();

        {
            Xvi {
                stations: stations,
                shots: shots,
                sketchlines: sketchlines,
                bounds: bounds,
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::Rule;
        use super::XVIParser;
        use super::parse_file;
        use crate::pest::*;

        // #[test]
        // fn parse() {
        //     parse_file("/Users/md/Documents/AKKH-Vermessungen/windloch/scraps/windloch2021_g27_g122_g203_g204_p.xvi".to_string());
        //     ()
        // }

        #[test]
        fn successful_parse() {
            // NUMBER
            parses_to! {
                parser: XVIParser,
                input:  "1.0",
                rule:   Rule::NUMBER,
                tokens: [NUMBER( 0, 3)]
            };

            // XVIgrids
            parses_to! {
                parser: XVIParser,
                input:  "XVIgrids {1.0 m}\n",
                rule:   Rule::xvigrids,
                tokens: [xvigrids(0, 16, [NUMBER(10,  13), ])]
            };
            parses_to! {
                parser: XVIParser,
                input:  "XVIgrids {1.0 m}",
                rule:   Rule::xvigrids,
                tokens: [xvigrids(0, 16, [NUMBER(10,  13), ])]

            };

            // XVIstations
            parses_to! {
                parser: XVIParser,
                input:  "{1111.22 1533.50 2.75}",
                rule:   Rule::station,
                tokens: [station(0, 22, [COORDINATE(1, 16, [NUMBER(1, 8), NUMBER(9, 16)]),  NAME(17, 21)])]

            };
            parses_to! {
                parser: XVIParser,
                input:  "XVIstations {\n    {1037.95 236.26 2.54}\n    {1107.91 337.68 2.55}\n}",
                rule:   Rule::xvistations,
                tokens: [
                    xvistations( 0,  67,[
                        station( 18, 39,
                        [COORDINATE(19, 33, [NUMBER(19, 26), NUMBER(27, 33)]),  NAME(34, 38)]
                   ),
                      station( 44, 65,
                    [COORDINATE(45, 59, [NUMBER(45, 52), NUMBER(53, 59)]),  NAME(60, 64)]
                   ),
                ])]
            };

            // XVIshots
            parses_to! {
            parser: XVIParser,
            input:  "{3032.80 830.51 3033.19 841.42}",
            rule:   Rule::shot,
            tokens: [
                shot(0, 31, [
                    COORDINATE(1, 15, [NUMBER(1, 8),
                    NUMBER(9, 15)]),
                    COORDINATE(16, 30, [NUMBER(16, 23),
                    NUMBER(24, 30)])
                 ])]
            };
            parses_to! {
                parser: XVIParser,
                input:  "XVIshots {\n    {3032.80 830.51 3033.19 841.42}\n    {3033.19 841.42 2974.53 894.33}\n}",
                rule:   Rule::xvishots,
                tokens: [xvishots(0, 84,[
                    shot(15, 46, [
                        COORDINATE(16, 30, [ NUMBER(16, 23),
                        NUMBER(24, 30)]),
                        COORDINATE(31, 45, [NUMBER(31, 38),
                        NUMBER(39, 45)])
                    ]),
                    shot(51, 82, [
                        COORDINATE(52, 66, [NUMBER(52, 59),
                        NUMBER(60, 66)]),
                        COORDINATE(67, 81, [NUMBER(67, 74),
                        NUMBER(75, 81)])
                    ])
                ])
                ]
            };

            // XVIsketchlines
            parses_to! {
            parser: XVIParser,
            input:  "{brown  2932.87 685.63  2928.15 686.22}",
            rule:   Rule::scetchline,
            tokens: [
                scetchline(0,39, [
                    COLOR(1, 6),
                    COORDINATE(8, 22, [NUMBER(8, 15), NUMBER(16, 22)]),
                    COORDINATE(24, 38, [NUMBER(24, 31), NUMBER(32, 38)])
                ])]
            };
            parses_to! {
                parser: XVIParser,
                input:  "XVIsketchlines {\n    {brown  2.3 2.6  7 3  1.81 4.17}\n    {black  39.37 6.61}\n}",
                rule:   Rule::xvisketchlines,
                tokens: [xvisketchlines(0, 79, [
                    scetchline(21,53, [
                        COLOR(22, 27),
                        COORDINATE(29, 36, [NUMBER(29, 32), NUMBER(33, 36)]),
                        COORDINATE(38, 41, [NUMBER(38, 39), NUMBER(40, 41)]),
                        COORDINATE(43, 52, [NUMBER(43, 47), NUMBER(48, 52)]),
                    ]),
                    scetchline(58,77, [
                        COLOR(59, 64),
                        COORDINATE(66, 76, [NUMBER(66, 71), NUMBER(72, 76)]),
                    ])

                ])]
            }
        }
        // set XVIgrids {1.0 m}
        // set XVIstations {
        //   {1037.95 236.26 2.54}
        // }
        //   set XVIshots {
        //     {3032.80 830.51 3033.19 841.42}
        // }
        // set XVIsketchlines {
        //     {orange  2.36 2.60  7.87 13.39  11.81 414.17  14.17 4.17}
        // }
        // set XVIgrid {738.9763779527558 -109.84251968503936 39.370078740157474 0.0 0.0 39.370078740157474 84 93}
        // fn unsuccessful_parse() {
        //     assert!(super::XVIParser::parse(super::Rules::encoding, "-128.0"));
        // }
    }
    //    let unparsed_file = fs::read_to_string(
    //         "/Users/md/Documents/AKKH-Vermessungen/windloch/scraps/windloch2021_g27_g122_g203_g204.th2",
    //     )
    //     .expect("cannot read file");

    //     let file = TH2Parser::parse(Rule::file, &unparsed_file)
    //     .expect("unsuccessful parse") // unwrap the parse result
    //     .next().unwrap(); // get and unwrap the `file` rule; never fails
}
