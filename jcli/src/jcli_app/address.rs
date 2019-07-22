use chain_addr::{AddressReadable, Discrimination, Kind};
use chain_crypto::bech32::Bech32 as _;
use chain_crypto::{AsymmetricPublicKey, Ed25519, PublicKey};
use jcli_app::utils::key_parser::parse_pub_key;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "address", rename_all = "kebab-case")]
pub enum Address {
    /// start jormungandr service and start participating to the network
    Info(InfoArgs),

    /// create an address from the single public key. This address does
    /// not have delegation
    Single(SingleArgs),

    /// create an address from the the single public key
    Account(AccountArgs),
}

#[derive(StructOpt)]
pub struct InfoArgs {
    /// An address, in bech32 format, to display the content
    /// and info that can be extracted from
    #[structopt(name = "ADDRESS")]
    address: AddressReadable,
}

#[derive(StructOpt)]
pub struct SingleArgs {
    /// A public key in bech32 encoding with the key type prefix
    #[structopt(name = "PUBLIC_KEY", parse(try_from_str = "parse_pub_key"))]
    key: PublicKey<Ed25519>,

    /// A public key in bech32 encoding with the key type prefix
    #[structopt(name = "DELEGATION_KEY", parse(try_from_str = "parse_pub_key"))]
    delegation: Option<PublicKey<Ed25519>>,

    /// set the human readable prefix for the address
    #[structopt(long = "prefix")]
    prefix: String,

    /// set the discrimination type to testing (default is production)
    #[structopt(long = "testing")]
    testing: bool,
}

#[derive(StructOpt)]
pub struct AccountArgs {
    /// A public key in bech32 encoding with the key type prefix
    #[structopt(name = "PUBLIC_KEY", parse(try_from_str = "parse_pub_key"))]
    key: PublicKey<Ed25519>,

    /// set the human readable prefix for the address
    #[structopt(long = "prefix")]
    prefix: String,

    /// set the discrimination type to testing (default is production)
    #[structopt(long = "testing")]
    testing: bool,
}

custom_error! {pub Error
    MultisigAddressNotSupported = "multisig addresses are not supported",
}

impl Address {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Address::Info(info_args) => address_info(&info_args.address)?,
            Address::Single(single_args) => {
                if let Some(delegation) = single_args.delegation {
                    mk_delegation(single_args.key, single_args.prefix, single_args.testing, delegation)
                } else {
                    mk_single(single_args.key, single_args.prefix, single_args.testing)
                }
            }
            Address::Account(account_args) => mk_account(account_args.key, account_args.prefix, account_args.testing),
        }
        Ok(())
    }
}

fn address_info(address: &AddressReadable) -> Result<(), Error> {
    let chain_addr::Address(discrimination, kind) = address.to_address();
    match discrimination {
        Discrimination::Production => {
            println!("discrimination: production");
        }
        Discrimination::Test => {
            println!("discrimination: testing");
        }
    }

    match kind {
        Kind::Single(single) => println!("public key: {}", single.to_bech32_str()),
        Kind::Account(account) => println!("account: {}", account.to_bech32_str()),
        Kind::Multisig(_) => return Err(Error::MultisigAddressNotSupported),
        Kind::Group(pubk, groupk) => {
            println!("public key: {}", pubk.to_bech32_str());
            println!("group key:  {}", groupk.to_bech32_str());
        }
    }
    Ok(())
}

fn mk_single(s: PublicKey<Ed25519>, prefix: String, testing: bool) {
    mk_address_1(s, prefix, testing, Kind::Single)
}

fn mk_delegation(s: PublicKey<Ed25519>, prefix: String, testing: bool, d: PublicKey<Ed25519>) {
    mk_address_2(s, d, prefix, testing, Kind::Group)
}

fn mk_account(s: PublicKey<Ed25519>, prefix: String, testing: bool) {
    mk_address_1(s, prefix, testing, Kind::Account)
}

fn mk_discrimination(testing: bool) -> Discrimination {
    if testing {
        Discrimination::Test
    } else {
        Discrimination::Production
    }
}

fn mk_address(discrimination: Discrimination, prefix: String, kind: Kind) {
    let address = chain_addr::Address(discrimination, kind);
    println!(
        "{}",
        AddressReadable::from_address(&prefix, &address).to_string()
    );
}

fn mk_address_1<A, F>(s: PublicKey<A>, prefix: String, testing: bool, f: F)
where
    F: FnOnce(PublicKey<A>) -> Kind,
    A: AsymmetricPublicKey,
{
    let discrimination = mk_discrimination(testing);
    let kind = f(s);
    mk_address(discrimination, prefix, kind);
}

fn mk_address_2<A1, A2, F>(s: PublicKey<A1>, d: PublicKey<A2>, prefix: String, testing: bool, f: F)
where
    F: FnOnce(PublicKey<A1>, PublicKey<A2>) -> Kind,
    A1: AsymmetricPublicKey,
    A2: AsymmetricPublicKey,
{
    let discrimination = mk_discrimination(testing);
    let kind = f(s, d);
    mk_address(discrimination, prefix, kind);
}
