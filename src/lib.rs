#![recursion_limit = "512"]

use itertools::Itertools;
use rust_bcf::{BcfRecord, BcfRecords, Record};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use ybc;
use ybc::TileCtx::{Ancestor, Child, Parent};
use ybc::TileSize::Four;
use yew::prelude::*;
use yew::services::reader::FileData;
use yew::web_sys::{console, File, FileReader};
use yew::Component;

struct BcfStatsApp {
    link: ComponentLink<Self>,
    records: Option<Vec<BcfRecord>>,
    file: Option<File>,
}

enum Msg {
    ClearFile,
    Update,
    SelectFile(Option<File>),
}

#[wasm_bindgen]
extern "C" {
    type Buffer;
}

// #[wasm_bindgen]
// pub fn read_records(file: &str) {// file_input: web_sys::HtmlInputElement) {
//     // //Check the file list from the input
//     // let filelist = file_input
//     //     .files()
//     //     .expect("Failed to get filelist from File Input!");
//     // //Do not allow blank inputs
//     // if filelist.length() < 1 {
//     //     // alert("Please select at least one file.");
//     //     return;
//     // }
//     // if filelist.get(0) == None {
//     //     // alert("Please select a valid file");
//     //     return;
//     // }
//     //
//     // let file = filelist.get(0).expect("Failed to get File from filelist!");
//
//     let file_reader: web_sys::FileReader = match web_sys::FileReader::new() {
//         Ok(f) => f,
//         Err(e) => {
//             // alert("There was an error creating a file reader");
//             web_sys::FileReader::new().expect("")
//         }
//     };
//
//     let fr_c = file_reader.clone();
//     // create onLoadEnd callback
//     let onloadend_cb = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
//         let array = js_sys::Uint8Array::new(&fr_c.result().unwrap());
//         let len = array.byte_length() as usize;
//         let data: Vec<u8> = array.to_vec();
//         let records = BcfRecords::new(data.as_slice());
//         // here you can for example use the received image/png data
//     }) as Box<dyn Fn(web_sys::ProgressEvent)>);
//
//     file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
//     file_reader
//         .read_as_array_buffer(&file)
//         .expect("blob not readable");
//     onloadend_cb.forget();
// }

use lazy_static::lazy_static;
use std::sync::Mutex;
lazy_static! {
    static ref RECORDS: Arc<Mutex<Vec<BcfRecord>>> = Arc::new(Mutex::new(vec![]));
}

impl Component for BcfStatsApp {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            records: None,
            file: None,
        }
    }

    // https://gist.github.com/soaxelbrooke/0b808bc495a3e2618311600d08cdb381
    // and
    // https://gist.github.com/rparrett/2137f02dcbc2bdc552d7ce3cdf3b68c7
    // and
    // https://github.com/rustwasm/wasm-bindgen/issues/2195
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SelectFile(file) => {
                self.file = file;
                let file_reader = FileReader::new().unwrap();
                file_reader
                    .read_as_array_buffer(&self.file.as_ref().unwrap())
                    .unwrap();

                let onload = Closure::wrap(Box::new(move |event: Event| {
                    let file_reader: FileReader = event.target().unwrap().dyn_into().unwrap();
                    let array = js_sys::Uint8Array::new(&file_reader.result().unwrap());
                    let data: Vec<u8> = array.to_vec();
                    let d = data.as_slice();
                    console::log_1(&format!("{:?}", d.len()).into());
                    let (reader, _format) = niffler::get_reader(Box::new(d)).unwrap();
                    let mut records = match BcfRecords::new(reader) {
                        Ok(records) => {
                            console::log_1(&format!("{:?}", "foo").into());
                            records.into_iter().collect::<Vec<_>>()
                        }
                        Err(e) => {
                            console::log_1(&format!("{:?}", "bar").into());
                            console::log_1(&format!("{:?}", e).into());
                            vec![]
                        }
                    };
                    console::log_1(&format!("{:?}", "baz").into());
                    RECORDS.lock().unwrap().clear();
                    RECORDS.lock().unwrap().append(&mut records);
                    // for record in records {
                    //     console::log_1(&format!("{:?}", std::str::from_utf8(&*record.id())).into());
                    // }
                }) as Box<dyn FnMut(_)>);

                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();

                true
            }
            Msg::ClearFile => {
                self.file = None;
                (*RECORDS.lock().unwrap()).clear();
                true
            }
            Msg::Update => true,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        match &self.file {
            Some(file) => html! {
                <>
                    <ybc::Container fluid=true>
                        <ybc::Tile ctx=Ancestor>
                            <ybc::Tile ctx=Parent vertical=true>
                                <ybc::Tile ctx=Child classes="box">
                                    <p>{ "Filename: "}<span>{ file.name() }</span></p>
                                    <p>{ "Last Modified: " }<span>{ file.last_modified() }</span></p>
                                    <p>{ "Size: " }<span>{ file.size() }</span></p>
                                </ybc::Tile>
                                <ybc::Tile ctx=Child classes="box">
                                    <input type="file" id="file-input" onchange=self.link.callback(|cd: ChangeData| {
                                        match cd {
                                            ChangeData::Files(file_list) => {
                                                // log::info!("File list: {:?}", file_list.get(0));
                                                Msg::SelectFile(file_list.get(0))
                                            },
                                            _ => Msg::ClearFile
                                        }
                                    })/>
                                    <button onclick=self.link.callback(|_| Msg::ClearFile)>{ "Clear" }</button>
                                    <button onclick=self.link.callback(|_| Msg::Update)>{ "Update" }</button>
                                </ybc::Tile>
                                <ybc::Tile ctx=Child classes="box">
                                    <table>
                                    <tr>
                                        <th>{"CHROM"}</th>
                                        <th>{"POS"}</th>
                                        <th>{"ID"}</th>
                                        <th>{"REF"}</th>
                                        <th>{"ALT"}</th>
                                        <th>{"QUAL"}</th>
                                    </tr>
                                    {
                                        for RECORDS.lock().unwrap().iter().map(|record| self.view_record(&record))
                                    }
                                    </table>
                                </ybc::Tile>
                            </ybc::Tile>
                        </ybc::Tile>
                    </ybc::Container>
                </>
            },
            None => html! {
                <>
                    <ybc::Tile ctx=Child classes="box">
                        <input type="file" id="file-input" onchange=self.link.callback(|cd: ChangeData| {
                            match cd {
                                ChangeData::Files(file_list) => {
                                    // log::info!("File list: {:?}", file_list.get(0));
                                    Msg::SelectFile(file_list.get(0))
                                },
                                _ => Msg::ClearFile
                            }
                        })/>
                        <button onclick=self.link.callback(|_| Msg::ClearFile)>{ "Clear" }</button>
                        <button onclick=self.link.callback(|_| Msg::Update)>{ "Update" }</button>
                    </ybc::Tile>
                </>
            },
        }
    }
}

impl BcfStatsApp {
    fn view_record(&self, record: &BcfRecord) -> Html {
        let record_id = record.id();
        let record_id = match &record_id {
            id if !id.is_empty() => std::str::from_utf8(&record_id).unwrap_or(".").clone(),
            _ => ".",
        };
        let alts = &record
            .alt_alleles()
            .iter()
            .map(|alt| std::str::from_utf8(alt).unwrap())
            .join(",");
        return html! {
            <tr>
            <td>{ record.chrom() }</td>
            <td>{ record.pos() }</td>
            <td>{ record_id }</td>
            <td>{ std::str::from_utf8(&record.ref_allele()).unwrap() }</td>
            <td>{ alts }</td>
            <td>{ format!("{:?}", record.qual().unwrap_or(0.0)) }</td>
            </tr>
        };
    }

    fn read_file(&mut self, file: FileData) {
        let data = file.content.as_slice();
        let records = BcfRecords::new(data).unwrap().collect::<Vec<_>>();
        self.records = Some(records);
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    yew::start_app::<BcfStatsApp>();
}
