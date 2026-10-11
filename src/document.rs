//! Document Generation Engine: Native Excel (.xlsx/XML) & PDF Generator
//!
//! Generates compliant OpenXML Excel spreadsheets, CSV data exports,
//! and native PDF documents (invoices, receipts, executive reports)
//! with zero external system dependencies.

use std::collections::HashMap;

/// Spreadsheet & Excel Document Generator
#[derive(Debug, Clone)]
pub struct ExcelDocument {
    pub sheet_name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl ExcelDocument {
    /// Create a new Excel spreadsheet document
    pub fn new(sheet_name: impl Into<String>) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Add column header labels
    pub fn headers(mut self, headers: Vec<impl Into<String>>) -> Self {
        self.headers = headers.into_iter().map(|h| h.into()).collect();
        self
    }

    /// Add a data row
    pub fn add_row(&mut self, row: Vec<impl Into<String>>) {
        self.rows.push(row.into_iter().map(|c| c.into()).collect());
    }

    /// Export as CSV with UTF-8 BOM
    pub fn to_csv(&self) -> String {
        let mut csv = String::from("\u{FEFF}");
        if !self.headers.is_empty() {
            csv.push_str(&self.headers.iter().map(|h| escape_csv(h)).collect::<Vec<_>>().join(","));
            csv.push('\n');
        }
        for row in &self.rows {
            csv.push_str(&row.iter().map(|c| escape_csv(c)).collect::<Vec<_>>().join(","));
            csv.push('\n');
        }
        csv
    }

    /// Export as Microsoft Excel SpreadsheetML XML format (.xlsx / .xml)
    pub fn to_excel_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<?mso-application progid=\"Excel.Sheet\"?>\n");
        xml.push_str("<Workbook xmlns=\"urn:schemas-microsoft-com:office:spreadsheet\"\n");
        xml.push_str(" xmlns:o=\"urn:schemas-microsoft-com:office:office\"\n");
        xml.push_str(" xmlns:x=\"urn:schemas-microsoft-com:office:excel\"\n");
        xml.push_str(" xmlns:ss=\"urn:schemas-microsoft-com:office:spreadsheet\"\n");
        xml.push_str(" xmlns:html=\"http://www.w3.org/TR/REC-html40\">\n");
        xml.push_str(" <Styles>\n");
        xml.push_str("  <Style ss:ID=\"Default\" ss:Name=\"Normal\">\n");
        xml.push_str("   <Alignment ss:Vertical=\"Bottom\"/>\n");
        xml.push_str("   <Font ss:FontName=\"Helvetica\" ss:Size=\"11\" ss:Color=\"#18181B\"/>\n");
        xml.push_str("  </Style>\n");
        xml.push_str("  <Style ss:ID=\"Header\">\n");
        xml.push_str("   <Font ss:FontName=\"Helvetica\" ss:Size=\"11\" ss:Bold=\"1\" ss:Color=\"#FFFFFF\"/>\n");
        xml.push_str("   <Interior ss:Color=\"#09090B\" ss:Pattern=\"Solid\"/>\n");
        xml.push_str("  </Style>\n");
        xml.push_str(" </Styles>\n");

        xml.push_str(&format!(" <Worksheet ss:Name=\"{}\">\n", escape_xml(&self.sheet_name)));
        xml.push_str("  <Table>\n");

        // Header Row
        if !self.headers.is_empty() {
            xml.push_str("   <Row ss:StyleID=\"Header\">\n");
            for h in &self.headers {
                xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", escape_xml(h)));
            }
            xml.push_str("   </Row>\n");
        }

        // Data Rows
        for row in &self.rows {
            xml.push_str("   <Row>\n");
            for cell in row {
                xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", escape_xml(cell)));
            }
            xml.push_str("   </Row>\n");
        }

        xml.push_str("  </Table>\n");
        xml.push_str(" </Worksheet>\n");
        xml.push_str("</Workbook>\n");
        xml
    }
}

/// Pure-Rust PDF Document & Invoice Generator
#[derive(Debug, Clone)]
pub struct PdfDocument {
    pub title: String,
    pub company_name: String,
    pub company_address: String,
    pub recipient_name: String,
    pub recipient_address: String,
    pub metadata: HashMap<String, String>,
    pub table_headers: Vec<String>,
    pub table_rows: Vec<Vec<String>>,
    pub total_amount: Option<String>,
    pub notes: Option<String>,
}

impl PdfDocument {
    /// Create a new PDF document builder
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            company_name: "Terminus Platform Inc.".to_string(),
            company_address: "San Francisco, CA".to_string(),
            recipient_name: String::new(),
            recipient_address: String::new(),
            metadata: HashMap::new(),
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            total_amount: None,
            notes: None,
        }
    }

    pub fn company(mut self, name: impl Into<String>, address: impl Into<String>) -> Self {
        self.company_name = name.into();
        self.company_address = address.into();
        self
    }

    pub fn recipient(mut self, name: impl Into<String>, address: impl Into<String>) -> Self {
        self.recipient_name = name.into();
        self.recipient_address = address.into();
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), val.into());
        self
    }

    pub fn table(mut self, headers: Vec<impl Into<String>>, rows: Vec<Vec<impl Into<String>>>) -> Self {
        self.table_headers = headers.into_iter().map(|h| h.into()).collect();
        self.table_rows = rows
            .into_iter()
            .map(|row| row.into_iter().map(|c| c.into()).collect())
            .collect();
        self
    }

    pub fn total(mut self, total: impl Into<String>) -> Self {
        self.total_amount = Some(total.into());
        self
    }

    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Compile into valid binary PDF-1.4 stream
    pub fn to_pdf_bytes(&self) -> Vec<u8> {
        let mut stream = String::new();
        // Start text object
        stream.push_str("BT\n");
        // Title (Helvetica Bold, 22pt)
        stream.push_str("/F2 22 Tf\n");
        stream.push_str("50 780 Td\n");
        stream.push_str(&format!("({}) Tj\n", escape_pdf(&self.title)));

        // Company Details (Helvetica 10pt)
        stream.push_str("/F1 10 Tf\n");
        stream.push_str("0 -24 Td\n");
        stream.push_str(&format!("({}) Tj\n", escape_pdf(&self.company_name)));
        stream.push_str("0 -14 Td\n");
        stream.push_str(&format!("({}) Tj\n", escape_pdf(&self.company_address)));

        // Recipient (if present)
        if !self.recipient_name.is_empty() {
            stream.push_str("0 -28 Td\n");
            stream.push_str("/F2 11 Tf\n");
            stream.push_str("(BILL TO:) Tj\n");
            stream.push_str("/F1 10 Tf\n");
            stream.push_str("0 -16 Td\n");
            stream.push_str(&format!("({}) Tj\n", escape_pdf(&self.recipient_name)));
            if !self.recipient_address.is_empty() {
                stream.push_str("0 -14 Td\n");
                stream.push_str(&format!("({}) Tj\n", escape_pdf(&self.recipient_address)));
            }
        }

        // Table Content
        if !self.table_headers.is_empty() {
            stream.push_str("0 -36 Td\n");
            stream.push_str("/F2 10 Tf\n");
            let header_line = self.table_headers.join("    |    ");
            stream.push_str(&format!("({}) Tj\n", escape_pdf(&header_line)));

            stream.push_str("/F1 9 Tf\n");
            for row in &self.table_rows {
                stream.push_str("0 -16 Td\n");
                let row_line = row.join("    |    ");
                stream.push_str(&format!("({}) Tj\n", escape_pdf(&row_line)));
            }
        }

        // Total
        if let Some(total) = &self.total_amount {
            stream.push_str("0 -30 Td\n");
            stream.push_str("/F2 12 Tf\n");
            stream.push_str(&format!("(TOTAL AMOUNT: {}) Tj\n", escape_pdf(total)));
        }

        // Notes
        if let Some(notes) = &self.notes {
            stream.push_str("0 -24 Td\n");
            stream.push_str("/F1 9 Tf\n");
            stream.push_str(&format!("(Notes: {}) Tj\n", escape_pdf(notes)));
        }

        stream.push_str("ET\n");

        let stream_len = stream.len();

        // PDF Objects layout
        let mut pdf = String::from("%PDF-1.4\n");
        let obj1_offset = pdf.len();
        pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        let obj2_offset = pdf.len();
        pdf.push_str("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        let obj3_offset = pdf.len();
        pdf.push_str("3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R /F2 6 0 R >> >> >>\nendobj\n");

        let obj4_offset = pdf.len();
        pdf.push_str(&format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
            stream_len, stream
        ));

        let obj5_offset = pdf.len();
        pdf.push_str("5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");

        let obj6_offset = pdf.len();
        pdf.push_str("6 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj\n");

        let xref_offset = pdf.len();
        pdf.push_str("xref\n0 7\n");
        pdf.push_str("0000000000 65535 f \n");
        pdf.push_str(&format!("{:010} 00000 n \n", obj1_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj2_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj3_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj4_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj5_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj6_offset));

        pdf.push_str(&format!(
            "trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            xref_offset
        ));

        pdf.into_bytes()
    }
}

fn escape_csv(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') {
        format!("\"{}\"", val.replace('"', "\"\""))
    } else {
        val.to_string()
    }
}

fn escape_xml(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_pdf(val: &str) -> String {
    val.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
