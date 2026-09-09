"""FIX-GG02 stage mismatches must fail static host validation."""
import finstack_chart as c
c.aes().x(c.stat_expr('Mean'))
c.stat_aes().y(c.source_expr('y'))
c.bin_aes().y(c.stat_expr('Count'))
c.scale_aes().size(c.source_expr('y'))
c.source_expr('x').add(c.stat_expr('Mean'))
