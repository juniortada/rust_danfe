// DANFE da NF-e, A4 retrato: MOC 7.0 Anexo II (Anexo III.02) com o leiaute da NT 2026.010.
#let data = json("data.json")
#let e = data.emitter
#let serif = ("Times New Roman", "Liberation Serif")

// Anexo II §3.6.2: margens de 0,2 a 0,8 cm.
#let margin = 5mm
#let header-height = if data.key_alphanumeric { 67mm } else { 54mm }
#let footer-height = 21mm

#let shown(value) = if value == none { "" } else { value }
// §3.7.1: descritivo dos blocos, negrito, caixa alta, ≥ 5 pt.
#let block-title(name) = block(above: 2.5pt, below: 1.2pt, text(size: 5pt, weight: "bold", upper(name)))
// §3.7.3 (descritivo ≥ 6 pt, caixa alta) e §3.7.9 (conteúdo ≥ 10 pt).
#let field(name, value, size: 10pt, bold: false, at: left) = block(
  width: 100%, inset: (x: 2pt, top: 1.5pt, bottom: 2pt), breakable: false,
  {
    text(size: 6pt, upper(name))
    linebreak()
    align(at, text(size: size, weight: if bold { "bold" } else { "regular" }, shown(value)))
  },
)
#let fields(columns, ..cells) = table(columns: columns, inset: 0pt, ..cells.pos())

#set text(font: serif, size: 10pt, lang: "pt", region: "br", hyphenate: false)
#set par(leading: 0.3em, spacing: 0.3em, justify: false)
#set table(stroke: 0.5pt)
#show table: set block(spacing: 0pt)

// Anexo II §3: em homologação, "SEM VALOR FISCAL" em marca d'água destacada.
#let watermark = if data.homologation {
  rotate(-35deg, align(center, text(fill: rgb(200, 0, 0, 60), weight: "bold")[
    #text(size: 46pt)[SEM VALOR FISCAL] \
    #text(size: 18pt)[EMITIDA EM AMBIENTE DE HOMOLOGAÇÃO]
  ]))
}

#let emitter-address = {
  let a = e.address
  if a != none [
    #a.street #if a.district != none [– #a.district] \
    #shown(a.city)/#shown(a.state) #if a.zip_code != none [– CEP #a.zip_code] \
    #if a.phone != none [Fone: #a.phone]
  ]
}

// Anexo II §3.5 e §3.10.2: identificação em todas as folhas, com o número da folha.
#let identification(page-number, pages) = {
  table(
    columns: (74mm, 34mm, 1fr), rows: 38mm, inset: 3pt,
    align: (left + top, center + top, left + top),
    {
      if data.logo != none { image(data.logo, height: 13mm, fit: "contain") }
      // §3.7.6: dados do emitente em negrito; razão social ≥ 12 pt, demais ≥ 8 pt.
      text(size: 12pt, weight: "bold", shown(e.name))
      if e.trade_name != none { linebreak(); text(size: 8pt, weight: "bold", e.trade_name) }
      linebreak()
      text(size: 8pt, weight: "bold", emitter-address)
    },
    align(center, {
      // §3.7.4.
      text(size: 12pt, weight: "bold")[DANFE]
      linebreak()
      text(size: 8pt)[DOCUMENTO AUXILIAR DA NOTA FISCAL ELETRÔNICA]
      v(2pt)
      grid(
        columns: (auto, auto), column-gutter: 4pt, align: horizon,
        align(left, text(size: 8pt)[0 - ENTRADA \ 1 - SAÍDA]),
        box(stroke: 0.5pt, inset: (x: 4pt, y: 2pt), text(size: 10pt, weight: "bold", data.direction)),
      )
      v(2pt)
      text(size: 10pt, weight: "bold")[Nº #data.number_dotted \ SÉRIE #data.series \ FOLHA #page-number/#pages]
    }),
    {
      if not data.key_alphanumeric {
        block(width: 100%, below: 3pt, image("chave.svg", width: 100%, height: 11mm, fit: "stretch"))
      }
      text(size: 6pt)[CHAVE DE ACESSO]
      linebreak()
      // §3.7.5: chave em negrito.
      text(size: 10pt, weight: "bold", data.access_key)
      v(3pt)
      text(size: 8pt)[Consulta de autenticidade no portal nacional da NF-e www.nfe.fazenda.gov.br/portal ou no site da Sefaz Autorizadora]
    },
  )
  if data.key_alphanumeric {
    // NT Conjunta 2025.001 §6: chave alfanumérica com no mínimo 11,5 cm.
    table(columns: 1fr, inset: (y: 1.5mm), align(center, image("chave.svg", width: 140mm, height: 10mm, fit: "stretch")))
  }
  fields((1fr, 92mm),
    field("Natureza da operação", data.operation),
    field("Protocolo de autorização de uso", data.protocol, at: center),
  )
  fields((1fr, 1fr, 1fr),
    field("Inscrição estadual", e.state_registration),
    field("Inscrição estadual do substituto tributário", e.st_registration),
    field("CNPJ / CPF", e.document),
  )
}

// Anexo II §3.3.1: sem formulário de segurança, o canhoto pode ir para a extremidade inferior.
#let receipt = {
  line(length: 100%, stroke: (thickness: 0.5pt, dash: "dashed"))
  v(1mm)
  table(
    columns: (1fr, 42mm), inset: 0pt,
    {
      block(inset: (x: 2pt, y: 2pt), text(size: 6pt)[RECEBEMOS DE #upper(shown(e.name)) OS PRODUTOS E/OU SERVIÇOS CONSTANTES DA NOTA FISCAL ELETRÔNICA INDICADA AO LADO])
      table(columns: (38mm, 1fr), inset: 0pt, stroke: (top: 0.5pt, rest: none),
        table.vline(x: 1, stroke: 0.5pt),
        block(height: 9mm, field("Data de recebimento", none)),
        block(height: 9mm, field("Identificação e assinatura do recebedor", none)),
      )
    },
    align(center + horizon, text(size: 10pt, weight: "bold")[NF-e \ Nº #data.number_dotted \ SÉRIE #data.series]),
  )
}

#set page(
  paper: "a4",
  margin: (left: margin, right: margin, top: margin + header-height, bottom: margin + footer-height),
  header-ascent: 0pt,
  footer-descent: 0pt,
  header: context {
    let pages = counter(page).final().first()
    let page-number = counter(page).get().first()
    block(height: header-height, align(bottom, identification(page-number, pages)))
  },
  footer: context {
    if counter(page).get().first() == 1 { block(height: footer-height, align(bottom, receipt)) }
  },
  background: watermark,
)

// NT 2026.010 §4.2: CRT e área reservada ao tipo de regime de apuração do IBS e da CBS.
#v(1.5mm)
#fields((1fr, 1fr),
  field("Código do regime tributário", e.tax_regime),
  field("Tipo de regime de apuração do IBS e da CBS", none),
)

#let r = data.recipient
#let ra = if r == none { none } else { r.address }
#let pick(container, key) = if container == none { none } else { container.at(key) }

#block-title("Destinatário / Remetente")
#fields((1fr, 45mm, 30mm),
  field("Nome / Razão social", pick(r, "name")),
  field("CNPJ / CPF", pick(r, "document"), bold: true),
  field("Data da emissão", data.issued_on),
)
#fields((1fr, 45mm, 24mm, 30mm),
  field("Endereço", pick(ra, "street")),
  field("Bairro / Distrito", pick(ra, "district")),
  field("CEP", pick(ra, "zip_code")),
  field("Data da entrada / saída", data.exit_on, bold: true),
)
#fields((1fr, 32mm, 10mm, 37mm, 30mm),
  field("Município", pick(ra, "city")),
  field("Fone / Fax", pick(ra, "phone")),
  field("UF", pick(ra, "state")),
  field("Inscrição estadual", pick(r, "state_registration")),
  field("Hora da entrada / saída", data.exit_time, bold: true),
)

// Anexo II §3.3.2: o quadro de fatura pode ser suprimido quando não há duplicatas.
#if data.duplicates.len() > 0 {
  block-title("Fatura / Duplicatas")
  table(
    columns: (1fr,) * 3, inset: (x: 2pt, y: 1.5pt),
    ..data.duplicates.map(d => grid(
      columns: (1fr, 1fr, 1fr),
      [#text(size: 6pt)[NÚMERO] \ #shown(d.number)],
      [#text(size: 6pt)[VENCIMENTO] \ #shown(d.due_date)],
      [#text(size: 6pt)[VALOR] \ #d.amount],
    )),
    ..range(calc.rem(3 - calc.rem(data.duplicates.len(), 3), 3)).map(_ => []),
  )
}

#let t = data.totals
#block-title("Total dos produtos e total da nota")
#fields((1fr,) * 5,
  field("Valor total dos produtos", t.products, at: right),
  field("Valor do frete", t.freight, at: right),
  field("Valor do seguro", t.insurance, at: right),
  field("Desconto", t.discount, at: right),
  field("Outras despesas acessórias", t.other, at: right),
)
#fields(1fr, field("Valor total da nota", t.total, at: right, bold: true))

#block-title("Total do ICMS / IPI")
#fields((1fr,) * 5,
  field("Base de cálculo do ICMS", t.icms_base, at: right),
  field("Valor do ICMS", t.icms, at: right),
  field("Base de cálculo do ICMS ST", t.st_base, at: right),
  field("Valor do ICMS ST", t.st, at: right),
  field("Valor do IPI", t.ipi, at: right),
)
// NT 2026.010 §4.4: campos facultativos só quando existem no XML.
#if (t.fcp, t.fcp_st, t.difal_destination, t.fcp_destination).any(v => v != none) {
  fields((1fr,) * 4,
    field("Valor do FCP", t.fcp, at: right),
    field("Valor do FCP retido por ST", t.fcp_st, at: right),
    field("Valor do DIFAL na UF de destino", t.difal_destination, at: right),
    field("Valor do FCP na UF de destino", t.fcp_destination, at: right),
  )
}
#if (t.icms_mono_quantity, t.icms_mono, t.icms_mono_withheld_quantity, t.icms_mono_withheld).any(v => v != none) {
  fields((1fr,) * 4,
    field("BC do ICMS monofásico", t.icms_mono_quantity, at: right),
    field("Valor do ICMS monofásico", t.icms_mono, at: right),
    field("BC do ICMS monofásico por retenção", t.icms_mono_withheld_quantity, at: right),
    field("Valor do ICMS monofásico por retenção", t.icms_mono_withheld, at: right),
  )
}

// NT 2026.010 §4.1.
#let rf = data.reform
#block-title("Total do IBS / CBS / IS")
#fields((1fr,) * 4,
  field("Valor da CBS", rf.cbs, at: right),
  field("Valor do IBS UF", rf.ibs_state, at: right),
  field("Valor do IBS município", rf.ibs_city, at: right),
  field("Valor do imposto seletivo", rf.selective, at: right),
)
#if rf.mono != none {
  fields((1fr,) * 4,
    field("Valor do IBS monofásico", rf.mono.ibs, at: right),
    field("Valor da CBS monofásica", rf.mono.cbs, at: right),
    field("Valor do IBS monofásico por retenção", rf.mono.ibs_withheld, at: right),
    field("Valor da CBS monofásica por retenção", rf.mono.cbs_withheld, at: right),
  )
}

// Anexo II §3.3.3: o ISSQN pode ser suprimido quando não se aplica.
#if data.issqn != none {
  block-title("Cálculo do ISSQN")
  fields((1fr,) * 4,
    field("Inscrição municipal", e.municipal_registration),
    field("Valor total dos serviços", data.issqn.services, at: right),
    field("Base de cálculo do ISSQN", data.issqn.base, at: right),
    field("Valor do ISSQN", data.issqn.value, at: right),
  )
}

#let tr = data.transport
#let carrier = tr.carrier
#let vehicle = tr.vehicle
#block-title("Transportador / Volumes transportados")
#fields((1fr, 36mm, 22mm, 20mm, 10mm, 36mm),
  field("Nome / Razão social", pick(carrier, "name")),
  field("Frete por conta", tr.mode, size: 8pt),
  field("Código ANTT", pick(vehicle, "antt")),
  field("Placa do veículo", pick(vehicle, "plate")),
  field("UF", pick(vehicle, "state")),
  field("CNPJ / CPF", pick(carrier, "document")),
)
#if carrier != none {
  fields((1fr, 60mm, 10mm, 36mm),
    field("Endereço", carrier.address),
    field("Município", carrier.city),
    field("UF", carrier.state),
    field("Inscrição estadual", carrier.state_registration),
  )
}
#for volume in tr.volumes {
  fields((1fr,) * 6,
    field("Quantidade", volume.quantity, at: right),
    field("Espécie", volume.kind),
    field("Marca", volume.brand),
    field("Numeração", volume.numbering),
    field("Peso bruto", volume.gross_weight, at: right),
    field("Peso líquido", volume.net_weight, at: right),
  )
}

// Anexo II §3.1.7, §3.2 e §3.7.7; NT 2026.010 §4.3. Conteúdo ≥ 6 pt.
#let tax-lines(lines) = grid(
  columns: (auto, 1fr), column-gutter: 2pt, row-gutter: 1.5pt,
  ..lines.filter(l => l != none).map(l => (l.at(0), align(right, l.at(1)))).flatten(),
)
#let percent(value) = if value == none { none } else { value + "%" }
#let item-row(i) = {
  let rtc = i.ibs_cbs
  let sel = i.selective
  let ipi = i.ipi
  (
    [#i.code \ #i.ncm],
    {
      i.description
      if i.additional != none { linebreak(); i.additional }
      if rtc != none { linebreak(); [cClassTrib #rtc.class] }
    },
    [#i.cst \ #i.cfop],
    align(right)[#i.quantity \ #i.unit],
    align(right)[#i.unit_price \ #i.discount],
    align(right, i.total),
    tax-lines((
      ("ICMS", i.icms_base),
      if rtc != none and rtc.base != none { ("IBS/CBS", rtc.base) },
      if sel != none { ("IS", sel.base) },
      if ipi != none { ("IPI", ipi.base) },
    )),
    tax-lines((
      ("ICMS", percent(i.icms_rate)),
      if rtc != none and rtc.cbs_rate != none { ("CBS", percent(rtc.cbs_rate)) },
      if rtc != none and rtc.ibs_state_rate != none { ("IBS UF", percent(rtc.ibs_state_rate)) },
      if rtc != none and rtc.ibs_city_rate != none { ("IBS MUN", percent(rtc.ibs_city_rate)) },
      if ipi != none { ("IPI", percent(ipi.rate)) },
      if sel != none { ("IS", percent(sel.rate)) },
    )),
    tax-lines((
      ("ICMS", i.icms_value),
      if rtc != none and rtc.cbs != none { ("CBS", rtc.cbs) },
      if rtc != none and rtc.ibs_state != none { ("IBS UF", rtc.ibs_state) },
      if rtc != none and rtc.ibs_city != none { ("IBS MUN", rtc.ibs_city) },
      if ipi != none { ("IPI", ipi.value) },
      if sel != none { ("IS", sel.value) },
    )),
  )
}
#let head(name) = text(size: 5pt, weight: "bold", upper(name))

#block-title("Dados dos produtos / serviços")
#{
  set text(size: 6pt)
  table(
    columns: (17mm, 1fr, 11mm, 16mm, 17mm, 16mm, 25mm, 21mm, 24mm),
    inset: (x: 1.5pt, y: 2pt),
    // §3.1.7: destaque divisório entre os itens.
    stroke: (x, y) => (left: 0.5pt, right: 0.5pt, top: 0.5pt, bottom: 0.5pt),
    table.header(
      head[Código / NCM], head[Descrição do produto / serviço], head[CST / CFOP], head[Qtd. / Un.],
      head[V. unit. / Desc.], head[V. total], head[Bases de cálculo], head[Alíquotas], head[Valor dos tributos],
    ),
    ..data.items.map(item-row).flatten(),
  )
}

// Anexo II §3.1.8: infAdFisco e infCpl.
#block-title("Dados adicionais")
#table(
  columns: (1fr, 62mm), inset: (x: 2pt, top: 1.5pt, bottom: 3pt),
  {
    text(size: 6pt)[INFORMAÇÕES COMPLEMENTARES]
    linebreak()
    set text(size: 6pt)
    if data.homologation [SEM VALOR FISCAL. ]
    if data.fiscal_info != none [#data.fiscal_info ]
    shown(data.additional)
    if t.approximate_taxes != none [ Valor aproximado dos tributos: R\$ #t.approximate_taxes.]
    v(12mm)
  },
  text(size: 6pt)[RESERVADO AO FISCO],
)
