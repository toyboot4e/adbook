# original: https://gist.github.com/mojavelinux/850373bb1b5a8e0334b4038e90617509

class PrismSyntaxHighlighter < Asciidoctor::SyntaxHighlighter::Base
    register_for 'prism'

    def format node, lang, opts
      opts[:transform] = proc do |pre, code|
        if node.attr? 'linenums', nil, false
          pre['class'] += ' line-numbers'
          if (start = node.attr 'start', nil, false)
            pre['data-start'] = start
          end
        end
        code['class'] = %(language-#{lang}) if lang
      end
      super
    end

    def docinfo? location
      location == :footer
    end

    # TODO add plugin to extract and restore callouts
    def docinfo location, doc, opts
      base_url = doc.attr 'prismdir', %(#{opts[:cdn_base_url]}/prism/1.15.0)
      slash = opts[:self_closing_tag_slash]
      unless (theme_name = doc.attr 'prism-style', 'prism') == 'prism'
        theme_name = %(prism-#{theme_name})
      end
      # TODO could load line-numbers plugin only if needed
      %(<link rel="stylesheet" href="#{base_url}/themes/#{theme_name}.min.css"#{slash}>
  <link rel="stylesheet" href="#{base_url}/plugins/line-numbers/prism-line-numbers.min.css"#{slash}>
  <script src="#{base_url}/prism.min.js"></script>
  <script src="#{base_url}/components/prism-ruby.min.js"></script>
  <script src="#{base_url}/plugins/line-numbers/prism-line-numbers.min.js"></script>)
    end
  end