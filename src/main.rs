use std::fmt::format;
use serde::Deserialize;
use serde::Serialize;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use arday11ChessLibrary::{convert_fen_to_vector, get_legal_moves_for_bishop, get_legal_moves_for_king, get_legal_moves_for_knight, get_legal_moves_for_pawn, get_legal_moves_for_queen, get_legal_moves_for_rook, get_pawn_capture_pos}; // Assuming this is a valid external library

#[derive(Serialize)]
#[derive(Debug)]
struct ResponseData {
    message: HashMap<String, Vec<Vec<usize>>>,
    status: u16,
}

#[derive(Deserialize)]
#[derive(Debug)]
struct ClientData {
    board: String,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn send_error_response(stream: &mut TcpStream, message: &str, status: u16) {
    let response = format!(
        "HTTP/1.1 {} Bad Request\r\n\
         Content-Type: text/plain\r\n\
         Access-Control-Allow-Origin: *\r\n\r\n\
         {}",
        status, message
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);

    if request.starts_with("OPTIONS") {
        // Handle preflight requests
        let response = "HTTP/1.1 204 No Content\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Access-Control-Allow-Methods: POST, OPTIONS\r\n\
                        Access-Control-Allow-Headers: Content-Type\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return; // Exit after handling OPTIONS
    }

    if request.starts_with("POST") {
        if let Some(pos) = request.find("\r\n\r\n") {
            let body = &request[pos + 4..]; // Extract the body part
            let body = body.trim(); // Trim any trailing whitespace
            //let body= format!("r#\"{}\"", body);
            let body = r#"{"board":"r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R"}"#;

            // Log the request body

            // Deserialize JSON into `ClientData`
            let client_data: Result<ClientData, serde_json::Error> = serde_json::from_str(&body);

            match client_data {
                Ok(data) => {
                    let board = convert_fen_to_vector(&data.board);

                    let mut legal_moves: HashMap<String, Vec<Vec<usize>>> = HashMap::new();

                    let mut rowIndex = 0;

                    for row in &board {
                        let mut colIndex = 0;

                        for square in row {
                            if (square == &'p' || square == &'P') {
                                let mut moves = get_legal_moves_for_pawn(&board, &Vec::from([rowIndex, colIndex]));

                                if (square == &'p') {
                                    let caputure_moves = get_pawn_capture_pos(&board, &Vec::from([rowIndex, colIndex]), 'p');

                                    for capture_move in caputure_moves {
                                        moves.push(capture_move)
                                    }
                                }

                                if (square == &'P') {
                                    let caputure_moves = get_pawn_capture_pos(&board, &Vec::from([rowIndex, colIndex]), 'P');

                                    for capture_move in caputure_moves {
                                        moves.push(capture_move)
                                    }
                                }

                                legal_moves.insert(format!("{}:{}", rowIndex, colIndex), moves);
                            }

                            else if (square == &'n' || square == &'N') {
                                let moves = get_legal_moves_for_knight(&board, &Vec::from([rowIndex, colIndex]));
                                legal_moves.insert(format!("{}:{}", rowIndex, colIndex), moves);
                            }

                            else if (square == &'r' || square == &'R') {
                                let moves = get_legal_moves_for_rook(&board, &Vec::from([rowIndex, colIndex]));
                                legal_moves.insert(format!("{}:{}", rowIndex, colIndex), moves);
                            }

                            else if (square == &'b' || square == &'B') {
                                let moves = get_legal_moves_for_bishop(&board, &Vec::from([rowIndex, colIndex]));
                                legal_moves.insert(format!("{}:{}", rowIndex, colIndex), moves);
                            }

                            else if (square == &'q' || square == &'Q') {
                                let moves = get_legal_moves_for_queen(&board, &Vec::from([rowIndex, colIndex]));
                                println!("{:?}", moves);
                                legal_moves.insert(format!("{}:{}", rowIndex, colIndex), moves);
                            }

                            else if (square == &'k' || square == &'K') {
                                let moves = get_legal_moves_for_king(&board, &Vec::from([rowIndex, colIndex]));
                                println!("{:?}", moves);
                                legal_moves.insert(format!("{}:{}", rowIndex, colIndex), moves);
                            }

                            colIndex += 1;
                        }

                        rowIndex += 1;
                    }

                    let response_data = ResponseData {
                        message: legal_moves,
                        status: 200,
                    };

                    let response_body = serde_json::to_string(&response_data).unwrap();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\n\
                             Content-Type: application/json\r\n\
                             Access-Control-Allow-Origin: *\r\n\
                             Access-Control-Allow-Methods: POST, OPTIONS\r\n\
                             Access-Control-Allow-Headers: Content-Type\r\n\r\n\
                             {}",
                        response_body
                    );

                    stream.write(response.as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
                Err(e) => {
                    println!("Failed to deserialize JSON: {}", e);
                    send_error_response(&mut stream, "Invalid JSON format", 400);
                }
            }
        }
    } else {
        // Handle other request methods (e.g., GET) if necessary
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\
                        Content-Type: text/plain\r\n\
                        Access-Control-Allow-Origin: *\r\n\r\n\
                        Only POST requests are supported.";
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

