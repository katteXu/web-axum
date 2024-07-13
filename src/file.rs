use std::path::PathBuf;

use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};

use crate::model::Record;

pub fn excel_to_record(excel_path: &PathBuf) -> Result<Vec<Record>, anyhow::Error> {
    let mut excel: Xlsx<_> = open_workbook(excel_path)?;
    let sheet_names = excel.sheet_names();

    if sheet_names.get(0).is_none() {
        anyhow::bail!("文件缺少sheet name");
    }

    let sheet_name = sheet_names.get(0).unwrap().to_string();

    let headers = [
        "域名",
        "建站年龄",
        "记录数",
        "开始时间",
        "结束时间",
        "标题",
        "语言",
        "评分",
        "DNS",
        "注册商",
        "注册商地址",
        "注册人",
        "Email",
        "注册时间",
        "到期时间",
        "更新时间",
        "备案状态",
        "备案时间",
        "备案主体",
        "备案类型",
        "备案号",
        "备案名",
    ];

    let range = excel.worksheet_range(&sheet_name)?;

    let iter = RangeDeserializerBuilder::with_headers(&headers).from_range(&range)?;

    let result = iter
        .map(|record| {
            let record: Record = record.unwrap();
            record
        })
        .collect::<Vec<_>>();

    Ok(result)
}
