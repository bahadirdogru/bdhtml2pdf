# Run with BDHTML2PDF_BINARY pointing to a built bdhtml2pdf:
#   ruby -Ilib test/test_bdhtml2pdf.rb
require "minitest/autorun"
require "tmpdir"
require "bdhtml2pdf"

PAGE = '<!DOCTYPE html><p id="p"></p><script>p.textContent = "Merhaba İğdır"</script>'.freeze

class LocalTest < Minitest::Test
  def test_pdf
    pdf = Bdhtml2pdf.convert(PAGE, page_size: "A5", title: "Fatura")
    assert pdf.start_with?("%PDF-")
    assert_includes pdf, "419.52"
  end

  def test_html_shows_that_scripts_ran_and_png
    assert_includes Bdhtml2pdf.convert(PAGE, format: :html).force_encoding("UTF-8"), '<p id="p">Merhaba İğdır'
    assert Bdhtml2pdf.convert(PAGE, format: :png, width: 300, height: 200).start_with?("\x89PNG".b)
  end

  def test_pdf_a
    assert_includes Bdhtml2pdf.convert("<title>T</title><p>x</p>", pdf_a: "2b"), "<pdfaid:part>2</pdfaid:part>"
  end

  def test_flags
    assert_equal ["--disable-javascript", "--zoom=2"],
                 Bdhtml2pdf.flags(disable_javascript: true, quiet: false, title: nil, zoom: 2)
  end

  def test_relative_files_load_from_base_dir
    Dir.mktmpdir do |d|
      File.write(File.join(d, "s.js"), 'document.body.textContent = "from s.js"')
      html = Bdhtml2pdf.convert('<body><script src="s.js"></script>', format: :html, base_dir: d)
      assert_includes html, "from s.js"
    end
  end

  def test_errors_carry_the_message
    e = assert_raises(Bdhtml2pdf::Error) { Bdhtml2pdf.convert("<p>x</p>", page_size: "Z9") }
    assert_match(/unknown page size/, e.message)
    assert_raises(ArgumentError) { Bdhtml2pdf.convert("<p>x</p>", format: :docx) }
  end
end

class ServerTest < Minitest::Test
  def setup
    @io = IO.popen([Bdhtml2pdf.binary, "serve", "--listen", "127.0.0.1:0", "--jobs", "2", "--max-memory", "200"],
                   err: %i[child out])
    url = @io.gets[%r{http://\S+}]
    @client = Bdhtml2pdf::Client.new(url)
  end

  def teardown
    Process.kill("KILL", @io.pid)
    @io.close
  end

  def test_server
    assert_match(/\Aok bdhtml2pdf/, @client.health)
    assert_includes @client.convert(PAGE, page_size: "A5"), "419.52"
    assert_includes @client.convert(PAGE, format: :html, disable_javascript: true), '<p id="p"></p>'
    e = assert_raises(Bdhtml2pdf::Error) { @client.convert("<p>x</p>", max_memory: 0) }
    assert_equal 400, e.status
    e = assert_raises(Bdhtml2pdf::Error) { @client.convert('<script>let s = "x"; for (;;) s = s + s;</script>') }
    assert_equal 507, e.status
  end
end
