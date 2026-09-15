// Reference farver byte conversion; independently covered by alpha-byte-rounding.json.
'use strict';
exports.alphaByte = function(value) {
  if (!Number.isFinite(value)) return Number.isNaN(value) ? 255 : 0;
  const scaled = Math.max(0, Math.min(1, value)) * 255;
  const lower = Math.floor(scaled), fraction = scaled - lower;
  return fraction === 0.5 ? lower + (lower % 2) : Math.round(scaled);
};
