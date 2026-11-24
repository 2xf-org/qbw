//! Preferences, the file's version, what QuickBooks worked out itself, users.

use crate::company::books::literal::row;
use crate::company::books::table::Table;

pub(super) fn tables() -> Vec<Table> {
    vec![
        preferences(),
        preference_values(),
        version(),
        facts(),
        saves(),
    ]
}

const NAMES: &[(i32, &str)] = &[
    (1, "COMPANYNAME"),
    (2, "LEGALNAME"),
    (3, "LEGALADDR1"),
    (4, "LEGALCITY"),
    (5, "LEGALSTATE"),
    (6, "LEGALZIP"),
    (7, "LEGALCOUNTRY"),
    (8, "ADDR"),
    (9, "COMPANYPHONENUM"),
    (10, "COMPANYEMAIL"),
    (11, "COMPANYWEBSITE"),
    (12, "FISCALBEGIN"),
    (13, "TAXBEGIN"),
    (14, "HOMECURRENCY"),
    (15, "USEMULTICURRENCY"),
    (16, "AGEFROMTXDATE"),
    (17, "CASHBASIS"),
    (18, "USEANUMBERS"),
    (19, "USECLASSES"),
    (20, "INVENTORY"),
    (21, "SALESTAXENABLED"),
    (22, "M_ENABLESALESORDER"),
    (23, "M_AVAILISONHANDMINUSONSO"),
    (24, "INCLUDEBUILDSINQTYAVAIL"),
    (25, "DEFAULTTAXITEM"),
    (26, "CURRENT_CLOSING_DATE"),
    (27, "STARTDATE"),
    (28, "CREATETIME"),
    (29, "SERIAL_NUMBER_TRACKING"),
    (30, "SHOWLOWESTSUB"),
];

fn preferences() -> Table {
    let table = Table::new(
        "cf_pref_attribute",
        &["
            preference_id integer key
            preference_name varchar(2048)
        "],
    );
    let names = NAMES.iter().map(|&(id, name)| {
        row! { "preference_id" => id, "preference_name" => name }
    });
    table.rows(Vec::new(), names)
}

/// The values of the whole company, by preference. The address has a row for
/// each of its lines.
const VALUES: &[(i32, &str)] = &[
    (1, "Harbor Valves"),
    (2, "Harbor Valves Ltd"),
    (3, "1 Quay Street"),
    (4, "Northport"),
    (5, "NP"),
    (6, "10001"),
    (7, "US"),
    (8, "1 Quay Street"),
    (8, "Northport, NP 10001"),
    (9, "555-0100"),
    (10, "office@harbor-valves.example"),
    (11, "harbor-valves.example"),
    (12, "1"),
    (13, "1"),
    (14, "1"),
    (15, "1"),
    (16, "0"),
    (17, "0"),
    (18, "1"),
    (19, "0"),
    (20, "1"),
    (21, "1"),
    (22, "1"),
    (23, "1"),
    (24, "1"),
    (25, "503"),
    (26, "12/31/2022"),
    (27, "2023/01/01"),
    (28, "1672565400"),
];

fn preference_values() -> Table {
    let table = Table::new(
        "cf_pref_attribute_value",
        &["
            pref_attribute_value_id integer key
            preference_id integer -> cf_pref_attribute
            user_name varchar(128) null
            preference_value varchar(32000) null
        "],
    );
    let values = (1..).zip(VALUES).map(|(id, &(preference, value))| {
        row! {
            "pref_attribute_value_id" => id,
            "preference_id" => preference,
            "preference_value" => value,
        }
    });
    table
        .rows(Vec::new(), values)
        // The setting of one user, whose name is in hexadecimal.
        .row(row! {
            "pref_attribute_value_id" => 101,
            "preference_id" => 30,
            "user_name" => "41646d696e",
            "preference_value" => "1",
        })
        .row(row! { "pref_attribute_value_id" => 102, "preference_id" => 29 })
}

fn version() -> Table {
    Table::new(
        "CF_SYSINFO",
        &["
            sysinfo_pk integer key
            sysinfo_name varchar(256)
            sysinfo_value varchar(256)
        "],
    )
    .row(row! { "sysinfo_pk" => 1, "sysinfo_name" => "DBMajorVersion", "sysinfo_value" => "34" })
    .row(row! { "sysinfo_pk" => 2, "sysinfo_name" => "DBReleaseNumber", "sysinfo_value" => "10" })
    .row(row! { "sysinfo_pk" => 3, "sysinfo_name" => "FileIsSample", "sysinfo_value" => "0" })
}

fn facts() -> Table {
    Table::new(
        "abmc_cf_facts",
        &["
            fact_id integer key
            fact_key varchar(128)
            fact_value varchar(4096) null
        "],
    )
    .row(row! {
        "fact_id" => 1,
        "fact_key" => "QBF_NetIncomePnLAccrualInLastFiscalYear",
        "fact_value" => "130.00",
    })
    .row(row! { "fact_id" => 2, "fact_key" => "MultiCurrencyUsed", "fact_value" => "true" })
}

/// The audit trail, as far as it names the users.
fn saves() -> Table {
    Table::new(
        "abmc_audit_header",
        &["
            transaction_id integer key
            revision_num integer key
            modtimestamp_tms integer null
            mod_user_id integer null
            mod_user_name varchar(128) null
            delete_tms integer null
            delete_user_id integer null
            delete_user_name varchar(128) null
        "],
    )
    .row(row! {
        "transaction_id" => -2147483548,
        "revision_num" => 1,
        "mod_user_id" => 1,
        "mod_user_name" => "Admin",
    })
}
