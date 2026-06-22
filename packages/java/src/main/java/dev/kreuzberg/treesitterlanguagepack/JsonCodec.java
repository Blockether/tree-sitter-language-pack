package dev.kreuzberg.treesitterlanguagepack;

import charred.CanonicalStrings;
import charred.CharReader;
import charred.JSONReader;
import charred.JSONWriter;
import java.io.StringWriter;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.BiConsumer;
import java.util.function.Function;
import org.jspecify.annotations.Nullable;

/**
 * Reflection-free JSON codec for the Rust FFI boundary, backed by charred
 * (com.cnuernber/charred) rather than Jackson databind.
 *
 * <p>The FFI exchanges JSON: configs are serialized to JSON before the downcall
 * and result structs are returned as JSON. charred parses to plain {@link Map}/
 * {@link List} trees (no reflection), and the typed DTOs are then built via
 * their generated public {@code Builder}s and enum {@code fromValue} factories —
 * so deserialization needs NO runtime reflection metadata, which is what makes
 * it safe and cheap under GraalVM native-image (Jackson databind reflecting over
 * every record was the liability this replaces).
 */
final class JsonCodec {
  private JsonCodec() {}

  // ----------------------------------------------------------------- charred IO

  /** Parse JSON into a plain tree: Map&lt;String,Object&gt; / List / String / Long / Double / Boolean / null. */
  static @Nullable Object parse(final String json) {
    try (JSONReader r = new JSONReader(
        JSONReader.defaultDoubleParser,
        JSONReader.mutableArrayReader,
        JSONReader.mutableObjReader,
        JSONReader.defaultEOFFn,
        new CanonicalStrings())) {
      r.beginParse(new CharReader(json));
      return r.readObject();
    } catch (Exception e) {
      throw new IllegalStateException("charred JSON parse failed: " + e.getMessage(), e);
    }
  }

  /** Serialize a plain tree (Map / List / String / Number / Boolean / null) to a JSON string. */
  static String write(final @Nullable Object value) {
    final StringWriter sw = new StringWriter();
    final BiConsumer<JSONWriter, Object> consumer = JsonCodec::writeValue;
    try (JSONWriter w = new JSONWriter(sw, false, false, false, null, consumer)) {
      w.writeObject(value);
    } catch (Exception e) {
      throw new IllegalStateException("charred JSON write failed: " + e.getMessage(), e);
    }
    return sw.toString();
  }

  @SuppressWarnings({"unchecked", "rawtypes"})
  private static void writeValue(final JSONWriter w, final @Nullable Object o) {
    try {
      if (o == null) {
        w.w.write("null");
      } else if (o instanceof Map<?, ?> m) {
        w.writeMap((java.util.Iterator) m.entrySet().iterator());
      } else if (o instanceof List<?> l) {
        w.writeArray((java.util.Iterator) l.iterator());
      } else if (o instanceof CharSequence cs) {
        w.writeString(cs);
      } else if (o instanceof Boolean b) {
        w.w.write(b ? "true" : "false");
      } else if (o instanceof Number n) {
        w.writeNumber(n);
      } else {
        w.writeString(o.toString());
      }
    } catch (Exception e) {
      throw new IllegalStateException("charred write value failed: " + e.getMessage(), e);
    }
  }

  // ------------------------------------------------------------- map accessors

  @SuppressWarnings("unchecked")
  private static @Nullable Map<String, Object> asMap(final @Nullable Object o) {
    return (Map<String, Object>) o;
  }

  @SuppressWarnings("unchecked")
  private static @Nullable List<Object> asList(final @Nullable Object o) {
    return (List<Object>) o;
  }

  private static @Nullable String str(final Map<String, Object> m, final String k) {
    final Object v = m.get(k);
    return v == null ? null : v.toString();
  }

  private static long lng(final Map<String, Object> m, final String k) {
    final Object v = m.get(k);
    return v == null ? 0L : ((Number) v).longValue();
  }

  private static boolean bool(final Map<String, Object> m, final String k) {
    final Object v = m.get(k);
    return v != null && (Boolean) v;
  }

  private static @Nullable List<String> strList(final Map<String, Object> m, final String k) {
    final List<Object> l = asList(m.get(k));
    if (l == null) {
      return null;
    }
    final List<String> out = new ArrayList<>(l.size());
    for (final Object o : l) {
      out.add(o == null ? null : o.toString());
    }
    return out;
  }

  private static <T> @Nullable List<T> mapList(
      final Map<String, Object> m, final String k, final Function<Map<String, Object>, T> f) {
    final List<Object> l = asList(m.get(k));
    if (l == null) {
      return null;
    }
    final List<T> out = new ArrayList<>(l.size());
    for (final Object o : l) {
      out.add(f.apply(asMap(o)));
    }
    return out;
  }

  // ------------------------------------------------------------- typed readers

  static @Nullable List<String> readStringList(final String json) {
    final List<Object> l = asList(parse(json));
    if (l == null) {
      return List.of();
    }
    final List<String> out = new ArrayList<>(l.size());
    for (final Object o : l) {
      out.add(o == null ? null : o.toString());
    }
    return out;
  }

  static ByteRange readByteRange(final String json) {
    final Map<String, Object> m = asMap(parse(json));
    return ByteRange.builder().withStart(lng(m, "start")).withEnd(lng(m, "end")).build();
  }

  static Point readPoint(final String json) {
    return point(asMap(parse(json)));
  }

  static ProcessResult readProcessResult(final String json) {
    return procResult(asMap(parse(json)));
  }

  private static @Nullable Point point(final @Nullable Map<String, Object> m) {
    if (m == null) {
      return null;
    }
    return Point.builder().withRow(lng(m, "row")).withColumn(lng(m, "column")).build();
  }

  private static @Nullable Span span(final @Nullable Map<String, Object> m) {
    if (m == null) {
      return null;
    }
    return Span.builder()
        .withStartByte(lng(m, "start_byte"))
        .withEndByte(lng(m, "end_byte"))
        .withStartLine(lng(m, "start_line"))
        .withStartColumn(lng(m, "start_column"))
        .withEndLine(lng(m, "end_line"))
        .withEndColumn(lng(m, "end_column"))
        .build();
  }

  private static FileMetrics fileMetrics(final Map<String, Object> m) {
    return FileMetrics.builder()
        .withTotalLines(lng(m, "total_lines"))
        .withCodeLines(lng(m, "code_lines"))
        .withCommentLines(lng(m, "comment_lines"))
        .withBlankLines(lng(m, "blank_lines"))
        .withTotalBytes(lng(m, "total_bytes"))
        .withNodeCount(lng(m, "node_count"))
        .withErrorCount(lng(m, "error_count"))
        .withMaxDepth(lng(m, "max_depth"))
        .build();
  }

  private static StructureItem structureItem(final Map<String, Object> m) {
    final String kind = str(m, "kind");
    return StructureItem.builder()
        .withKind(kind == null ? null : StructureKind.fromValue(kind))
        .withName(str(m, "name"))
        .withVisibility(str(m, "visibility"))
        .withSpan(span(asMap(m.get("span"))))
        .withChildren(mapList(m, "children", JsonCodec::structureItem))
        .withDecorators(strList(m, "decorators"))
        .withDocComment(str(m, "doc_comment"))
        .withSignature(str(m, "signature"))
        .withBodySpan(span(asMap(m.get("body_span"))))
        .build();
  }

  private static ImportInfo importInfo(final Map<String, Object> m) {
    return ImportInfo.builder()
        .withSource(str(m, "source"))
        .withItems(strList(m, "items"))
        .withAlias(str(m, "alias"))
        .withIsWildcard(bool(m, "is_wildcard"))
        .withSpan(span(asMap(m.get("span"))))
        .build();
  }

  private static ExportInfo exportInfo(final Map<String, Object> m) {
    final String kind = str(m, "kind");
    return ExportInfo.builder()
        .withName(str(m, "name"))
        .withKind(kind == null ? null : ExportKind.fromValue(kind))
        .withSpan(span(asMap(m.get("span"))))
        .build();
  }

  private static CommentInfo commentInfo(final Map<String, Object> m) {
    final String kind = str(m, "kind");
    return CommentInfo.builder()
        .withText(str(m, "text"))
        .withKind(kind == null ? null : CommentKind.fromValue(kind))
        .withSpan(span(asMap(m.get("span"))))
        .withAssociatedNode(str(m, "associated_node"))
        .build();
  }

  private static DocSection docSection(final Map<String, Object> m) {
    return DocSection.builder()
        .withKind(str(m, "kind"))
        .withName(str(m, "name"))
        .withDescription(str(m, "description"))
        .build();
  }

  private static DocstringInfo docstringInfo(final Map<String, Object> m) {
    final String format = str(m, "format");
    return DocstringInfo.builder()
        .withText(str(m, "text"))
        .withFormat(format == null ? null : DocstringFormat.fromValue(format))
        .withSpan(span(asMap(m.get("span"))))
        .withAssociatedItem(str(m, "associated_item"))
        .withParsedSections(mapList(m, "parsed_sections", JsonCodec::docSection))
        .build();
  }

  private static SymbolInfo symbolInfo(final Map<String, Object> m) {
    final String kind = str(m, "kind");
    return SymbolInfo.builder()
        .withName(str(m, "name"))
        .withKind(kind == null ? null : SymbolKind.fromValue(kind))
        .withSpan(span(asMap(m.get("span"))))
        .withTypeAnnotation(str(m, "type_annotation"))
        .withDoc(str(m, "doc"))
        .build();
  }

  private static Diagnostic diagnostic(final Map<String, Object> m) {
    final String severity = str(m, "severity");
    return Diagnostic.builder()
        .withMessage(str(m, "message"))
        .withSeverity(severity == null ? null : DiagnosticSeverity.fromValue(severity))
        .withSpan(span(asMap(m.get("span"))))
        .build();
  }

  private static DataAttribute dataAttribute(final Map<String, Object> m) {
    return DataAttribute.builder()
        .withName(str(m, "name"))
        .withValue(str(m, "value"))
        .withSpan(span(asMap(m.get("span"))))
        .build();
  }

  private static DataNode dataNode(final Map<String, Object> m) {
    final String kind = str(m, "kind");
    return DataNode.builder()
        .withKind(kind == null ? null : DataNodeKind.fromValue(kind))
        .withKey(str(m, "key"))
        .withValue(str(m, "value"))
        .withAttributes(mapList(m, "attributes", JsonCodec::dataAttribute))
        .withChildren(mapList(m, "children", JsonCodec::dataNode))
        .withSpan(span(asMap(m.get("span"))))
        .build();
  }

  private static ChunkContext chunkContext(final Map<String, Object> m) {
    return ChunkContext.builder()
        .withLanguage(str(m, "language"))
        .withChunkIndex(lng(m, "chunk_index"))
        .withTotalChunks(lng(m, "total_chunks"))
        .withNodeTypes(strList(m, "node_types"))
        .withContextPath(strList(m, "context_path"))
        .withSymbolsDefined(strList(m, "symbols_defined"))
        .withComments(mapList(m, "comments", JsonCodec::commentInfo))
        .withDocstrings(mapList(m, "docstrings", JsonCodec::docstringInfo))
        .withHasErrorNodes(bool(m, "has_error_nodes"))
        .build();
  }

  private static CodeChunk codeChunk(final Map<String, Object> m) {
    final Object meta = m.get("metadata");
    return CodeChunk.builder()
        .withContent(str(m, "content"))
        .withStartByte(lng(m, "start_byte"))
        .withEndByte(lng(m, "end_byte"))
        .withStartLine(lng(m, "start_line"))
        .withEndLine(lng(m, "end_line"))
        .withMetadata(meta == null ? null : chunkContext(asMap(meta)))
        .build();
  }

  private static ProcessResult procResult(final Map<String, Object> m) {
    final Object metrics = m.get("metrics");
    final Object data = m.get("data");
    return ProcessResult.builder()
        .withLanguage(str(m, "language"))
        .withMetrics(metrics == null ? null : fileMetrics(asMap(metrics)))
        .withStructure(mapList(m, "structure", JsonCodec::structureItem))
        .withImports(mapList(m, "imports", JsonCodec::importInfo))
        .withExports(mapList(m, "exports", JsonCodec::exportInfo))
        .withComments(mapList(m, "comments", JsonCodec::commentInfo))
        .withDocstrings(mapList(m, "docstrings", JsonCodec::docstringInfo))
        .withSymbols(mapList(m, "symbols", JsonCodec::symbolInfo))
        .withDiagnostics(mapList(m, "diagnostics", JsonCodec::diagnostic))
        .withChunks(mapList(m, "chunks", JsonCodec::codeChunk))
        .withData(data == null ? null : dataNode(asMap(data)))
        .build();
  }

  // ------------------------------------------------------------- typed writers

  static String writeConfig(final ProcessConfig c) {
    final Map<String, Object> m = new LinkedHashMap<>();
    m.put("language", c.language());
    putIf(m, "structure", c.structure());
    putIf(m, "imports", c.imports());
    putIf(m, "exports", c.exports());
    putIf(m, "comments", c.comments());
    putIf(m, "docstrings", c.docstrings());
    putIf(m, "symbols", c.symbols());
    putIf(m, "diagnostics", c.diagnostics());
    putIf(m, "chunk_max_size", c.chunkMaxSize());
    putIf(m, "data_extraction", c.dataExtraction());
    return write(m);
  }

  static String writeConfig(final PackConfig c) {
    final Map<String, Object> m = new LinkedHashMap<>();
    putIf(m, "cache_dir", c.cacheDir() == null ? null : c.cacheDir().toString());
    putIf(m, "languages", c.languages());
    putIf(m, "groups", c.groups());
    return write(m);
  }

  static String writeStringList(final List<String> names) {
    return write(new ArrayList<Object>(names));
  }

  private static void putIf(final Map<String, Object> m, final String k, final @Nullable Object v) {
    if (v != null) {
      m.put(k, v);
    }
  }
}
