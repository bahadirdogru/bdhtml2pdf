require_relative "lib/bdhtml2pdf/version"

Gem::Specification.new do |s|
  s.name = "bdhtml2pdf"
  s.version = Bdhtml2pdf::VERSION
  s.summary = "HTML + JavaScript to PDF with bdhtml2pdf, from Ruby"
  s.description = "Runs the bdhtml2pdf program, or talks to a `bdhtml2pdf serve` server. No native extension."
  s.authors = ["Bahadır"]
  s.license = "GPL-3.0-or-later"
  s.homepage = "https://bdhtml2pdf.bahadirdogru.com"
  s.metadata = { "source_code_uri" => "https://github.com/bahadirdogru/bdhtml2pdf" }
  s.required_ruby_version = ">= 2.7"
  s.files = Dir["lib/**/*.rb"] + ["README.md"]
end
