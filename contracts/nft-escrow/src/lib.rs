#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct Offer<M: ManagedTypeApi> {
    pub creator: ManagedAddress<M>,
    pub nft: TokenIdentifier<M>,
    pub nonce: u64,
    pub wanted_nft: TokenIdentifier<M>,
    pub wanted_nonce: u64,
    pub wanted_address: ManagedAddress<M>,
}

#[multiversx_sc::contract]
pub trait NftEscrowContract {
    #[init]
    fn init(&self) {}

    #[payable("*")]
    #[endpoint]
    fn escrow(
        &self,
        wanted_nft: TokenIdentifier,
        wanted_nonce: u64,
        wanted_address: ManagedAddress,
    ) -> u32 {
        let payment = self.call_value().single_esdt();

        require!(
            payment.token_nonce > 0 && payment.amount == 1,
            "ESDT is not an NFT"
        );
        require!(wanted_nonce > 0, "Wanted ESDT is not an NFT");

        let creator = self.blockchain().get_caller();

        require!(
            creator != wanted_address,
            "Wanted address should not be the same as the caller"
        );

        let offer_id = self.last_offer_id().update(|v| {
            *v += 1;

            *v
        });

        self.created_offers(&creator).insert(offer_id);
        self.wanted_offers(&wanted_address).insert(offer_id);

        let offer = Offer {
            creator,
            nft: payment.token_identifier,
            nonce: payment.token_nonce,
            wanted_nft,
            wanted_nonce,
            wanted_address,
        };

        self.offers(offer_id).set(offer);

        offer_id
    }

    #[endpoint]
    fn cancel(&self, offer_id: u32) {
        let offers_mapper = self.offers(offer_id);

        require!(!offers_mapper.is_empty(), "Offer does not exist");

        let caller = self.blockchain().get_caller();

        let offer = offers_mapper.get();

        require!(
            offer.creator == caller,
            "Only the offer creator can cancel it"
        );

        self.created_offers(&caller).swap_remove(&offer_id);
        self.wanted_offers(&offer.wanted_address)
            .swap_remove(&offer_id);

        self.offers(offer_id).clear();

        self.tx()
            .to(&offer.creator)
            .payment(EsdtTokenPayment::new(
                offer.nft,
                offer.nonce,
                BigUint::from(1u64),
            ))
            .transfer();
    }

    #[payable("*")]
    #[endpoint]
    fn accept(&self, offer_id: u32) {
        let offers_mapper = self.offers(offer_id);

        require!(!offers_mapper.is_empty(), "Offer does not exist");

        let offer = offers_mapper.get();

        let caller = self.blockchain().get_caller();

        require!(offer.wanted_address == caller, "Can not accept this offer");

        let payment = self.call_value().single_esdt();

        require!(
            payment.token_identifier == offer.wanted_nft
                && payment.token_nonce == offer.wanted_nonce
                && payment.amount == 1,
            "NFT does not match"
        );

        self.created_offers(&offer.creator).swap_remove(&offer_id);
        self.wanted_offers(&offer.wanted_address)
            .swap_remove(&offer_id);

        self.offers(offer_id).clear();

        self.tx().to(&offer.creator).payment(payment).transfer();
        self.tx()
            .to(&offer.creator)
            .payment(EsdtTokenPayment::new(
                offer.nft,
                offer.nonce,
                BigUint::from(1u64),
            ))
            .transfer();
    }

    #[view(getCreatedOffers)]
    fn get_created_offers(
        &self,
        address: ManagedAddress,
    ) -> MultiValueEncoded<MultiValue2<u32, Offer<Self::Api>>> {
        let mut result = MultiValueEncoded::new();

        for offer_id in self.created_offers(&address).iter() {
            result.push(self.get_offer_result(offer_id));
        }

        result
    }

    #[view(getWantedOffers)]
    fn get_wanted_offers(
        &self,
        address: ManagedAddress,
    ) -> MultiValueEncoded<MultiValue2<u32, Offer<Self::Api>>> {
        let mut result = MultiValueEncoded::new();

        for offer_id in self.wanted_offers(&address).iter() {
            result.push(self.get_offer_result(offer_id));
        }

        result
    }

    fn get_offer_result(&self, offer_id: u32) -> MultiValue2<u32, Offer<Self::Api>> {
        let offer = self.offers(offer_id).get();

        MultiValue2::from((offer_id, offer))
    }

    #[view]
    #[storage_mapper("createdOffers")]
    fn created_offers(&self, address: &ManagedAddress) -> UnorderedSetMapper<u32>;

    #[view]
    #[storage_mapper("wantedOffers")]
    fn wanted_offers(&self, address: &ManagedAddress) -> UnorderedSetMapper<u32>;

    #[view]
    #[storage_mapper("offers")]
    fn offers(&self, id: u32) -> SingleValueMapper<Offer<Self::Api>>;

    #[storage_mapper("lastOfferId")]
    fn last_offer_id(&self) -> SingleValueMapper<u32>;
}
