## Flow
ak-p2p.1 connects to ak-signalv;
ak-p2p.1 sends identification data about itself (for now only "name"+"target");
ak-signalv saves a hashmap relating "name"<>identification;
ak-signalv checks if "target" exists in the hashmap -- it doesn't;
ak-signalv sends negative response to ak-p2p.1;
ak-signalv disconnects ak-p2p.1;
ak-p2p.1 waits for X secs for "target" to try connection;

ak-p2p.2 connects to ak-signalv;
ak-p2p.2 sends identification data about itself (for now only "name"+"target");
ak-signalv saves a hashmap relating "name"<>identification;
ak-signalv checks if "target" exists in the hashmap -- it does;
ak-signalv sends positive response to ak-p2p.2 with identification of target;
ak-signalv disconnects ak-p2p.2
ak-p2p.2 sends connection request to ak-p2p.1;

ak-p2p.1 receives connection request and try matching if name == target -- it does;
ak-p2p.1 accepts connection;
