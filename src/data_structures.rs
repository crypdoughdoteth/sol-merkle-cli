use anyhow::Result;
use clap::Args;
use ethers::{
    abi::{self, ethabi::Bytes, Address, Token},
    types::{H160, U256},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    hash::Hash,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    rc::Rc,
    str::FromStr,
    sync::Arc,
    thread,
};
#[derive(Clone, Debug)]
pub struct Tree {
    root: Node,
    neighbors: HashMap<Bytes, Bytes>,
}

impl Tree {
    pub(crate) fn new(mut self, leafs: &Vec<Leaf>) -> Tree {
        //returns new merkle tree constructed from leafs: Vec<Leaf>
        // ---------------------------------------
        //creates initial nodes for merkle tree
        //iterate through leafs
        //on each iteration, hash the data contained in the leaf with Keccak256
        //append new Node to children_nodes
        let mut nodes: Vec<Node> = Vec::new();
        let mut parent_nodes: Vec<Node> = Vec::new();
        let mut children_nodes: Vec<Node> = leafs
            .iter()
            .map(|x| {
                let x_hashed = Leaf::hash_leaf(&x);
                println!("child hash: {:?}", &x_hashed);
                let y = Node::new(x_hashed.into());
                nodes.push(y.clone());
                y
            })
            .collect();
        //check length of leaf nodes, if there is an odd number of leaf nodes, copy the odd element and push it to the array
        let len = &children_nodes.len();
        if len % 2 == 1 {
            children_nodes.push(children_nodes[len - 1].clone());
            println!("inserted child hash: {:?}", &children_nodes[len - 1].hash);
        }

        let mut n = children_nodes.len();
        //main loop, make sure there is at least two elemnts left, otherwise we've hit the root
        while n > 1 {
            let mut i = 0;
            //iterates over children_nodes to create new parent nodes
            while i < n - 1 {
                //link neighbors using hashmap, allows us to reconstruct the tree later with K:V pairs
                let cnode = Rc::new(children_nodes.clone());
                let node_one = &Rc::clone(&cnode)[i].hash;
                let node_two = &Rc::clone(&cnode)[i + 1].hash;
                self.neighbors.insert(node_one.clone(), node_two.clone());
                self.neighbors.insert(node_two.clone(), node_one.clone());
                //hash nodes together
                let new_node_hash = Node::hash_nodes(&[node_one.clone(), node_two.clone()]);

                println!("node hash: {:?}", &new_node_hash);
                let new_node = Node {
                    hash: new_node_hash,
                };
                parent_nodes.push(new_node.clone());
                nodes.push(new_node);

                //println!("node child left: {:#?}", &new_node.left_child);
                //println!("node child right: {:#?}", &new_node.right_child);

                //increment loop by 2
                i += 2;
            }
            //change level
            children_nodes = parent_nodes;
            parent_nodes = Vec::new();
            //divide # of starting nodes by 2, represents remaining population of nodes that can be hashed together
            n = n / 2;
            //if there is an odd number of nodes excluding the root, insert copy of the [len-1] node
            let length: usize = nodes.len();
            if n > 1 && n % 2 == 1 {
                children_nodes.push(children_nodes[children_nodes.len() - 1].clone());
                nodes.push(children_nodes[children_nodes.len() - 1].clone());
                self.neighbors.insert(
                    children_nodes[children_nodes.len() - 1].hash.clone(),
                    children_nodes[children_nodes.len() - 2].hash.clone(),
                );
                self.neighbors.insert(
                    children_nodes[children_nodes.len() - 2].hash.clone(),
                    children_nodes[children_nodes.len() - 1].hash.clone(),
                );
                println!("inserted node hash: {:#?}", nodes[length - 1].hash);
                n += 1;
            }
            if n == 1 {
                self.root = Node {
                    hash: nodes[length - 1].hash.clone(),
                };
                println!("root: {:#?}", self.root.hash);
            }
        }
        self
    }
    pub(crate) fn spawn() -> Tree {
        Tree {
            root: Node::new(Leaf::hash_leaf(&Leaf {
                address: "0x0000000000000000000000000000000000000000".to_owned(),
                amount: 0,
            })),
            neighbors: HashMap::new(),
        }
    }

    pub(crate) fn get_root(self) -> Bytes {
        self.root.hash
    }

    //use hashmap to locate neighbored elements and generate parent node hashes
    pub(crate) fn generate_proof(&self, leaf: &Leaf, index: usize) -> Option<Vec<Bytes>> {
        //strat 2.0: the leaf hash we receive MUST be located in the HashMap, else throw error, invalid leaf (or end of set).
        //retrieve the corresponding VALUE inside of the HashMap for leaf's hash
        //hash together the resulting hashes
        //lookup result and repeat process

        let leaf_hash: Bytes = Leaf::hash_leaf(&leaf);

        let mut idx = index;
        let mut proof_hashes: Vec<Bytes> = vec![];
        let mut current_hash = leaf_hash;
        let mut parent_node_hash: Bytes;
        loop {
            let neighbor_hash = self.neighbors.get(&current_hash);
            match neighbor_hash {
                Some(hash) => {
                    if idx % 2 == 0 {
                        parent_node_hash =
                            Node::hash_nodes(&[current_hash, (*hash.clone()).to_vec()]);
                    } else {
                        parent_node_hash =
                            Node::hash_nodes(&[(*hash.clone()).to_vec(), current_hash]);
                    }
                    proof_hashes.push((*hash).clone());
                    current_hash = parent_node_hash;
                    idx /= 2;
                }
                //if we cannot locate a value in the mapping given the key, break the loop and return the vector of hashes
                None => break,
            }
        }

        if proof_hashes.len() == 0 {
            return None;
        } else {
            Some(proof_hashes)
        }
    }

    // pub(crate) fn write_tree(self: Self, path: &str) -> Result<(), std::io::Error> {
    //     let file = File::create(path)?;
    //     let mut writer = BufWriter::new(file);
    //     let json = serde_json::to_string(&self)?;
    //     serde_json::to_writer(&mut writer, &json)?;
    //     writer.flush()?;
    //     Ok(())
    // }
    // pub(crate) fn overwrite_tree(self: Self, path: &str) -> Result<(), std::io::Error> {
    //     let file = File::open(path)?;
    //     file.set_len(0)?;
    //     let mut writer = BufWriter::new(file);
    //     let json = serde_json::to_string(&self)?;
    //     serde_json::to_writer(&mut writer, &json)?;
    //     writer.flush()?;
    //     Ok(())
    // }
    // pub(crate) fn read_tree(path: &str) -> Result<Tree, std::io::Error> {
    //     let file = File::open(path)?;
    //     let mut buf_reader = BufReader::new(file);
    //     let mut contents = String::new();
    //     let _ = buf_reader.read_to_string(&mut contents);
    //     let res: Tree = serde_json::from_str(&contents)?;
    //     Ok(res)
    // }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Node {
    hash: Bytes,
}

impl Node {
    fn new(h: Bytes) -> Node {
        Self { hash: h }
    }
    fn hash_nodes(nodes: &[Bytes; 2]) -> Bytes {
        let a = Token::Bytes(nodes[0].clone());
        let b = Token::Bytes(nodes[1].clone());
        let encode = abi::encode(&[a, b]);
        keccak256(encode).into()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Leaf {
    address: String,
    amount: u32,
}
impl Leaf {
    pub(crate) fn new(address: String, amount: u32) -> Self {
        Self { address, amount }
    }

    pub(crate) fn hash_leaf(input: &Leaf) -> Bytes {
        let addy = Token::Address(H160::from_str(&input.address).unwrap());
        let amount = Token::Uint(input.amount.into());
        Self::encode_hash(&[addy, amount])
    }
    pub(crate) fn encode_hash(args: &[abi::Token]) -> Bytes {
        keccak256(abi::encode(args)).into()
    }
}

fn verify_proof(merkle_root: Bytes, leaf: Leaf, hashes: Vec<Bytes>, index: usize) -> bool {
    let mut counter = 0;
    let mut idx = index;
    let mut hash = Leaf::hash_leaf(&leaf);
    let mut verified = false;
    loop {
        let proof_element = &hashes[counter];

        if idx % 2 == 0 {
            hash = Node::hash_nodes(&[hash, proof_element.clone()]);
        } else {
            hash = Node::hash_nodes(&[proof_element.clone(), hash]);
        }
        println!("Verifying...{:?}", &hash);

        if hash == merkle_root {
            verified = true;
            break;
        }
        if counter == hashes.len() - 1 {
            break;
        }
        counter += 1;
        idx /= 2;
    }

    verified
}

pub(crate) fn cli_gen_root(path: &PathBuf) -> Result<()> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(&file);
    let mut contents = String::new();
    let _ = buf_reader.read_to_string(&mut contents);
    let res: Vec<Leaf> = serde_json::from_str(&contents)?;
    let tr = Tree::spawn();
    let root = tr.new(&res).root.hash;
    println!("{:?}", &root);

    Ok(())
}

pub(crate) fn cli_gen_proof(path: &PathBuf, idx: Option<usize>) -> Result<()> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(&file);
    let mut contents = String::new();
    let _ = buf_reader.read_to_string(&mut contents);
    let res: Vec<Leaf> = serde_json::from_str(&contents)?;
    let tr = Tree::spawn();
    let tree = tr.new(&res);

    let proof_res = match idx {
        Some(i) => {
            let r = &mut tree.generate_proof(&res[i], i).unwrap();
            vec![Proof {
                root: r.pop().unwrap(),
                proof: r.clone(),
            }]
        }
        None => {
            let handles = thread::spawn(move || {
                let mut v = vec![];

                for j in 0..&res.len() - 1 {
                    let proof = &mut tree.generate_proof(&res[j], j).unwrap();
                    v.push(Proof {
                        root: proof.pop().unwrap(),
                        proof: proof.clone(),
                    });
                }
                v
            });
            handles.join().unwrap()
        }
    };
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let json = serde_json::to_string(&proof_res)?;
    serde_json::to_writer(&mut writer, &json)?;
    writer.flush()?;
    Ok(())
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Proof {
    root: Bytes,
    proof: Vec<Bytes>,
}
