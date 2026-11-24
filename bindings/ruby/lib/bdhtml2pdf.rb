# HTML + JavaScript to PDF with bdhtml2pdf.
#
#   Bdhtml2pdf.convert(html, page_size: "A5")             # runs the bdhtml2pdf program
#   Bdhtml2pdf::Client.new("http://127.0.0.1:7368").convert(html)  # `bdhtml2pdf serve`
#
# Options are the command-line flags in snake_case: page_size: "A5",
# margin_top: "20mm", pdf_a: "2b", disable_javascript: true.

require "net/http"
require "open3"
require "uri"
require_relative "bdhtml2pdf/version"

module Bdhtml2pdf
  FORMATS = %w[pdf png jpg html].freeze

  # A conversion failed. +status+ is the exit status or HTTP status.
  class Error < StandardError
    attr_reader :status

    def initialize(message, status = nil)
      super(message)
      @status = status
    end
  end

  module_function

  # Options as command-line flags. +true+ is a bare flag; +false+ and +nil+
  # leave the flag out. Values use --flag=value, so a value that starts with
  # a dash stays a value.
  def flags(options)
    options.filter_map do |key, value|
      next if value.nil? || value == false

      name = key.to_s.tr("_", "-")
      value == true ? "--#{name}" : "--#{name}=#{value}"
    end
  end

  def check_format(format)
    f = format.to_s.downcase
    f = "jpg" if f == "jpeg"
    raise ArgumentError, "unknown format #{format.inspect}; expected one of #{FORMATS.join(', ')}" unless FORMATS.include?(f)

    f
  end

  def binary(path = nil)
    found = path || ENV["BDHTML2PDF_BINARY"]
    return found if found

    exe = Gem.win_platform? ? "bdhtml2pdf.exe" : "bdhtml2pdf"
    ENV.fetch("PATH", "").split(File::PATH_SEPARATOR).each do |dir|
      p = File.join(dir, exe)
      return p if File.executable?(p)
    end
    raise Error, "bdhtml2pdf not found; install it, put it on PATH or set BDHTML2PDF_BINARY"
  end

  def clean(message, fallback)
    m = message.to_s.strip
    m = fallback if m.empty?
    m.sub(/\Aerror: /, "")
  end

  # Convert +html+ and return the PDF (or PNG, JPEG, HTML) bytes as a binary
  # String. +base_dir+ is where relative images, styles and scripts load
  # from; +binary+ is the path to bdhtml2pdf (default: $BDHTML2PDF_BINARY,
  # then PATH).
  def convert(html, format: "pdf", base_dir: nil, binary: nil, **options)
    cmd = [binary(binary), "-q", "--format=#{check_format(format)}", *flags(options), "-", "-"]
    opts = { binmode: true }
    opts[:chdir] = base_dir if base_dir
    out, err, status = Open3.capture3(*cmd, stdin_data: html.to_s, **opts)
    raise Error.new(clean(err.force_encoding("UTF-8"), "exit status #{status.exitstatus}"), status.exitstatus) unless status.success?

    out
  end

  # A client for `bdhtml2pdf serve`. The server loads nothing from its disk,
  # so a page carries its images and styles inline (data: URLs, <style>).
  class Client
    attr_reader :url

    def initialize(url = "http://127.0.0.1:7368", timeout: 60)
      @url = url.sub(%r{/+\z}, "")
      @timeout = timeout
    end

    def health
      request(Net::HTTP::Get.new(URI("#{@url}/health"))).strip
    end

    def convert(html, format: "pdf", **options)
      query = options.filter_map do |key, value|
        next if value.nil? || value == false

        [key.to_s.tr("_", "-"), value == true ? "true" : value.to_s]
      end
      uri = URI("#{@url}/#{Bdhtml2pdf.check_format(format)}")
      uri.query = URI.encode_www_form(query) unless query.empty?
      req = Net::HTTP::Post.new(uri, "Content-Type" => "text/html; charset=utf-8")
      req.body = html.to_s
      request(req)
    end

    private

    def request(req)
      uri = req.uri
      res = Net::HTTP.start(uri.host, uri.port, use_ssl: uri.scheme == "https",
                                                open_timeout: @timeout, read_timeout: @timeout) do |http|
        http.request(req)
      end
      unless res.is_a?(Net::HTTPSuccess)
        raise Error.new(Bdhtml2pdf.clean(res.body.to_s.force_encoding("UTF-8"), "HTTP #{res.code}"), res.code.to_i)
      end

      res.body.to_s.b
    end
  end
end
