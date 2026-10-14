#let data = json("data.json")
#set page(paper: "a4", margin: (x: 7mm, y: 7mm),
  background: if data.homologation { rotate(-35deg, text(size: 34pt, fill: rgb(220, 0, 0, 60))[SEM VALOR FISCAL]) })
#set text(font: "Libertinus Serif", size: 6.5pt, lang: "pt")
#set par(spacing: 0.25em, leading: 0.35em)
#let label(body) = text(size: 5pt, weight: "bold")[#upper(body)]
#let field(name, value) = box(width: 100%, inset: (x: 1.5pt, y: 1.5pt))[#label(name) \ #value]
#let section(title) = block(above: 3pt, below: 1pt)[#text(size: 6pt, weight: "bold")[#upper(title)]]
#let boxed(..cells, columns: none) = table(columns: columns, stroke: 0.4pt, inset: 0pt, ..cells)
#let emitter = data.emitter
#let address(a) = if a == none { "" } else { [#a.street — #a.district \ #a.city / #a.state — CEP #a.zip_code] }

#boxed(columns: (1fr,),
  field("Recebemos de " + emitter.name + " os produtos e/ou serviços constantes da nota fiscal indicada ao lado",
    grid(columns: (1fr, 2fr, auto), gutter: 4pt, [Data de recebimento], [Identificação e assinatura do recebedor], [NF-e Nº #data.number_padded Série #data.series])))
#v(2pt)
#boxed(columns: (2.2fr, 1fr, 2.4fr),
  box(inset: 3pt, height: 26mm)[
    #text(size: 9pt, weight: "bold")[#emitter.name] \
    #if emitter.trade_name != none [#emitter.trade_name \ ]
    #address(emitter.address)
    #if emitter.address.phone != none [ \ Fone: #emitter.address.phone]
  ],
  box(inset: 3pt, height: 26mm)[
    #align(center)[
      #text(size: 11pt, weight: "bold")[DANFE] \
      #text(size: 5.5pt)[Documento Auxiliar da Nota Fiscal Eletrônica] \
      #v(1pt)
      0 - Entrada \ 1 - Saída #h(3pt) #box(stroke: 0.4pt, inset: 2pt)[#data.direction] \
      #v(1pt)
      #text(weight: "bold")[Nº #data.number_padded] \
      #text(weight: "bold")[Série #data.series]
    ]
  ],
  box(inset: 3pt, height: 26mm)[
    #image("chave.svg", width: 100%, height: 10mm, fit: "stretch")
    #label("Chave de acesso") \
    #text(size: 7pt, weight: "bold")[#data.access_key] \
    #v(1pt)
    Consulta de autenticidade no portal nacional da NF-e www.nfe.fazenda.gov.br/portal ou no site da Sefaz autorizadora
  ],
)
#boxed(columns: (2.2fr, 1fr, 1.4fr),
  field("Natureza da operação", data.operation),
  field("Inscrição estadual", emitter.state_registration),
  field("Protocolo de autorização de uso", data.protocol),
)
#boxed(columns: (1fr, 1fr, 1fr), field("CNPJ", emitter.document), field("Inscrição estadual do subst. tributário", ""), field("Data de emissão", data.issued_at))

#section("Destinatário / remetente")
#if data.recipient != none {
  let r = data.recipient
  let a = r.address
  boxed(columns: (3fr, 1.4fr, 1fr),
    field("Nome / razão social", r.name), field("CNPJ / CPF", r.document), field("Data da emissão", data.issued_at),
  )
  boxed(columns: (3fr, 1.4fr, 1fr),
    field("Endereço", if a != none { a.street } else { "" }), field("Bairro / distrito", if a != none { a.district } else { "" }), field("CEP", if a != none { a.zip_code } else { "" }),
  )
  boxed(columns: (2fr, 0.5fr, 1fr, 1.4fr),
    field("Município", if a != none { a.city } else { "" }), field("UF", if a != none { a.state } else { "" }),
    field("Fone", if a != none and a.phone != none { a.phone } else { "" }), field("Inscrição estadual", if r.state_registration != none { r.state_registration } else { "" }),
  )
}

#if data.duplicates.len() > 0 [
  #section("Fatura / duplicatas")
  #boxed(columns: (1fr,) * calc.min(data.duplicates.len(), 6),
    ..data.duplicates.map(d => field("Nº " + d.number, [Venc. #d.due_date \ R\$ #d.amount])))
]

#section("Cálculo do imposto")
#let t = data.totals
#boxed(columns: (1fr,) * 6,
  field("Base de cálculo do ICMS", t.icms_base), field("Valor do ICMS", t.icms),
  field("Base de cálculo ICMS ST", t.st_base), field("Valor do ICMS ST", t.st),
  field("Valor total dos produtos", t.products), field("Valor aprox. dos tributos", if t.approximate_taxes != none { t.approximate_taxes } else { "" }),
  field("Valor do frete", t.freight), field("Valor do seguro", t.insurance),
  field("Desconto", t.discount), field("Outras despesas", t.other),
  field("Valor do IPI", t.ipi), field("Valor total da nota", text(weight: "bold")[#t.total]),
)

#section("Transportador / volumes transportados")
#boxed(columns: (1fr,), field("Frete por conta", data.freight_mode))

#section("Dados dos produtos / serviços")
#table(
  columns: (auto, 1fr, auto, auto, auto, auto, auto, auto, auto, auto, auto, auto),
  stroke: 0.4pt, inset: 2pt,
  align: (left, left, center, center, center, center, right, right, right, right, right, right),
  table.header(..("Código", "Descrição", "NCM", "CST", "CFOP", "Un.", "Qtde.", "V. unit.", "V. total", "Desc.", "BC ICMS", "V. ICMS").map(h => label(h))),
  ..data.items.map(i => (i.code, i.description, i.ncm, i.cst, i.cfop, i.unit, i.quantity, i.unit_price, i.total, i.discount, i.icms_base, i.icms_value)).flatten()
)

#section("Dados adicionais")
#boxed(columns: (2fr, 1fr),
  box(inset: 2pt, height: 22mm)[#label("Informações complementares") \ #if data.additional != none [#data.additional]
    #if data.homologation [ \ NF-E EMITIDA EM AMBIENTE DE HOMOLOGAÇÃO — SEM VALOR FISCAL]],
  box(inset: 2pt, height: 22mm)[#label("Reservado ao fisco")],
)
