/*
 * Loudness normalizer based on the EBU R128 standard
 *
 * Copyright (c) 2014, Alessandro Ghedini
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 *       notice, this list of conditions and the following disclaimer.
 *
 *     * Redistributions in binary form must reproduce the above copyright
 *       notice, this list of conditions and the following disclaimer in the
 *       documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
 * IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
 * LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
 * NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#pragma once

#include "src/lib.rs.h"
#ifdef CXXBRIDGE1_RUST_STRING

bool tag_write_mp3(Scan scan, bool do_album, bool extended, rust::String unit,
                   bool lowercase, bool strip, int id3v2version);
bool tag_clear_mp3(rust::String filee, bool strip, int id3v2version);

bool tag_write_flac(Scan scan, bool do_album, bool extended, rust::String unit);
bool tag_clear_flac(rust::String filee);

bool tag_write_ogg_vorbis(Scan scan, bool do_album, bool extended, rust::String unit);
bool tag_clear_ogg_vorbis(rust::String filee);

bool tag_write_ogg_flac(Scan scan, bool do_album, bool extended, rust::String unit);
bool tag_clear_ogg_flac(rust::String filee);

bool tag_write_ogg_speex(Scan scan, bool do_album, bool extended, rust::String unit);
bool tag_clear_ogg_speex(rust::String filee);

bool tag_write_ogg_opus(Scan scan, bool do_album, bool extended, rust::String unit);
bool tag_write_ogg_opus_non_standard(Scan scan, bool do_album, bool extended, rust::String unit);
bool tag_clear_ogg_opus(rust::String filee);

bool tag_write_mp4(Scan scan, bool do_album, bool extended, rust::String unit,
                   bool lowercase);
bool tag_clear_mp4(rust::String filee);

bool tag_write_asf(Scan scan, bool do_album, bool extended, rust::String unit,
                   bool lowercase);
bool tag_clear_asf(rust::String filee);

bool tag_write_wav(Scan scan, bool do_album, bool extended, rust::String unit,
                   bool lowercase, bool strip, int id3v2version);
bool tag_clear_wav(rust::String filee, bool strip, int id3v2version);

bool tag_write_aiff(Scan scan, bool do_album, bool extended, rust::String unit,
                    bool lowercase, bool strip, int id3v2version);
bool tag_clear_aiff(rust::String filee, bool strip, int id3v2version);

bool tag_write_wavpack(Scan scan, bool do_album, bool extended, rust::String unit,
                       bool lowercase, bool strip);
bool tag_clear_wavpack(rust::String filee, bool strip);

bool tag_write_ape(Scan scan, bool do_album, bool extended, rust::String unit,
                   bool lowercase, bool strip);
bool tag_clear_ape(rust::String filee, bool strip);

int gain_to_q78num(double gain);

int tag_version_major();
int tag_version_minor();
int tag_version_patch();

#endif