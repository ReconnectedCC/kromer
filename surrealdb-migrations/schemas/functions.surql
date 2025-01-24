DEFINE FUNCTION OVERWRITE fn::create_wallet($initial_balance: option<decimal>) {
LET $address = rand::string(10).lowercase();
LET $password = rand::string(16);
LET $hash = crypto::argon2::generate($password);
LET $wallet = (CREATE wallet CONTENT { address: $address, balance: $initial_balance OR 0f, hash: $hash });
RETURN { address: $address, password: $password, wallet: $wallet.first() };
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::create_wallet_ext($address: string, $hash: string, $initial_bal: decimal) {
    LET $wallet = (CREATE wallet CONTENT { address: $address, balance: $initial_bal OR 0f, hash: $hash });
    RETURN $wallet; 
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::transfer_balance($from: string, $to: string | record<name>, $amount: decimal) {
LET $from_wallet = (SELECT * FROM wallet WHERE address == $from).first();
LET $to_wallet = (SELECT * FROM wallet WHERE address == $to).first();
UPDATE $from_wallet SET balance -= $amount;
UPDATE $from_wallet SET total_out += $amount;
UPDATE $to_wallet SET balance += $amount;
UPDATE $to_wallet SET total_in += $amount;
} PERMISSIONS FULL;