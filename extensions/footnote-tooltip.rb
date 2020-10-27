# original: https://gist.github.com/sudofoobar/074f723b0e0d286a9012bcb6a786a400

# Asciidoctor-extension-footnote-tooltip.rb
# To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
# You should have received a copy of the CC0 Public Domain Dedication along with this software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

require 'asciidoctor/extensions'
require 'nokogumbo'

Asciidoctor::Extensions.register do
  postprocessor FootnoteTooltipPostProcessor
end

class FootnoteTooltipPostProcessor < Asciidoctor::Extensions::Postprocessor
  def process document, output
    if document.attributes.key?('noheader')
      outdoc = Nokogiri::HTML5.fragment(output)
    else
      outdoc = Nokogiri::HTML5(output)
    end

    foot_refs = outdoc.search('sup.footnote>a')
    for footnote in document.footnotes
      node = foot_refs[footnote.index.to_i-1]
      orig_str = node.serialize
      node['title'] = Nokogiri::HTML5.fragment(footnote.text).xpath('.//text()').text
      output = output.sub(orig_str, node.serialize)
    end
    output
  end
end