import AVFoundation
import Darwin
import Foundation

struct VerificationFailure: Error, CustomStringConvertible {
    let description: String
}

func require(_ condition: @autoclosure () -> Bool, _ message: String) throws {
    guard condition() else {
        throw VerificationFailure(description: message)
    }
}

func verifySyntheticWave(at path: String) throws {
    let url = URL(fileURLWithPath: path)
    let audioFile = try AVAudioFile(
        forReading: url,
        commonFormat: .pcmFormatInt16,
        interleaved: false
    )
    let format = audioFile.processingFormat

    try require(format.commonFormat == .pcmFormatInt16, "decoder did not produce signed 16-bit PCM")
    try require(format.channelCount == 1, "expected mono audio")
    try require(format.sampleRate == 8_000, "expected an 8 kHz sample rate")
    try require(audioFile.length == 800, "expected 800 decoded frames (100 ms at 8 kHz)")

    let capacity = AVAudioFrameCount(audioFile.length)
    guard let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: capacity) else {
        throw VerificationFailure(description: "could not allocate the decode buffer")
    }
    try audioFile.read(into: buffer)
    try require(buffer.frameLength == capacity, "decoder returned an unexpected frame count")

    guard let samples = buffer.int16ChannelData?[0] else {
        throw VerificationFailure(description: "decoder returned no mono sample data")
    }
    let period: [Int16] = [0, 8_192, 11_585, 8_192, 0, -8_192, -11_585, -8_192]
    for index in 0..<Int(buffer.frameLength) {
        try require(
            samples[index] == period[index % period.count],
            "decoded PCM sample mismatch at frame \(index)"
        )
    }
}

guard CommandLine.arguments.count == 2 else {
    fputs("usage: swift verify-synthetic-wav.swift <input.wav>\n", stderr)
    exit(EXIT_FAILURE)
}

do {
    try verifySyntheticWave(at: CommandLine.arguments[1])
    print("native AVFoundation decode matched all 800 synthetic PCM samples")
} catch {
    fputs("synthetic WAV decode check failed: \(error)\n", stderr)
    exit(EXIT_FAILURE)
}
