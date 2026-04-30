fn main() {
    vaultship_sdk::validate_or_exit(
        "/opt/vaultship/license.key",
        include_bytes!("../keys/vaultship.public.key"),
    );

    // Start your service runtime here.
}
