//! The Fintrix demo corpus: real RBI / FEMA provisions, chunked.

pub struct Chunk {
    pub src: &'static str,
    pub kw: &'static str,
    pub text: &'static str,
    pub ans: &'static str,
}

pub const CORPUS: &[Chunk] = &[
    Chunk { src: "RBI Master Direction · Liberalised Remittance Scheme", kw: "send money abroad remit remittance limit year annual foreign usd dollars how much",
        text: "Resident individuals may remit up to USD 250,000 per financial year (April–March) under LRS for permitted current or capital account transactions.",
        ans: "A resident individual can send up to USD 250,000 abroad in each financial year (April to March) under the Liberalised Remittance Scheme, for permitted current or capital account purposes. Remitting beyond that limit needs prior approval from the RBI." },
    Chunk { src: "FEMA, 1999 · Section 13 (Penalties)", kw: "penalty penalties fine violate violation contravention breach punishment",
        text: "A contravention can attract a penalty of up to three times the sum involved where quantifiable, or up to ₹2 lakh where it is not.",
        ans: "For a FEMA contravention the adjudicating authority can impose a penalty of up to three times the amount involved when that amount can be quantified, or up to ₹2 lakh when it cannot. A continuing contravention can attract an additional daily penalty." },
    Chunk { src: "FEMA, 1999 · Section 3", kw: "buy purchase foreign exchange currency who dealer authorised authorized bank money changer forex",
        text: "No person may deal in or transfer foreign exchange or a foreign security except through an authorised person, unless the Act permits it.",
        ans: "Foreign exchange has to be bought, sold or transferred through an authorised person, such as an authorised dealer bank or a licensed money changer, unless FEMA or its rules specifically permit otherwise." },
    Chunk { src: "RBI Master Direction · LRS (permitted transactions)", kw: "gift relative abroad nri non-resident donation family",
        text: "Gifts and donations to persons resident outside India are permitted for resident individuals, within the overall LRS limit.",
        ans: "Yes. A resident individual can gift money to a relative living abroad, but the gift counts toward the same USD 250,000 per financial year LRS limit as any other remittance." },
    Chunk { src: "RBI Master Direction · Know Your Customer", kw: "kyc account open opening identity verification documents due diligence customer",
        text: "Regulated entities must carry out customer due diligence, identifying the customer and verifying identity, before starting an account-based relationship.",
        ans: "Before opening an account, a bank or other regulated entity must complete customer due diligence: identify you and verify your identity with officially valid documents." },
    Chunk { src: "Foreign Exchange Management (Overseas Investment) Rules, 2022", kw: "invest overseas investment shares foreign stocks odi opi portfolio",
        text: "Overseas investment by persons resident in India, including direct and portfolio investment, is governed by the Overseas Investment Rules and Regulations, 2022.",
        ans: "Investing abroad is governed by the Overseas Investment Rules and Regulations, 2022. For a resident individual, overseas portfolio investment is routed through LRS and therefore sits inside the annual USD 250,000 limit." },
];

pub const SUGGESTIONS: &[&str] = &[
    "How much money can I send abroad in a year?",
    "What is the penalty for violating FEMA?",
    "Can I gift money to my relative in the US?",
    "Who can I buy foreign currency from?",
    "What is the capital of France?",
];
