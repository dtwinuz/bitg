cask "bitg" do
  version "0.3.1"
  sha256 "ec0d84412f1a5b440c41c4a17d1a12d8f76f73ec6ea32846df3a2c7961b207c4"

  url "https://github.com/dtwinuz/bitg/releases/download/v#{version}/bitg-#{version}-macos.pkg",
      verified: "github.com/dtwinuz/bitg/"
  name "bitg"
  desc "Uzbek input method for the new 2026 Latin alphabet"
  homepage "https://bitg.uz/"

  livecheck do
    url "https://github.com/dtwinuz/bitg"
    strategy :github_latest
  end

  depends_on :macos

  pkg "bitg-#{version}-macos.pkg"

  uninstall quit:    "uz.bitg.inputmethod.bitg",
            pkgutil: "uz.bitg.inputmethod.pkg",
            delete:  "/Library/Input Methods/bitg.app"

  zap trash: "~/Library/Application Support/bitg"

  caveats <<~EOS
    Log out and back in, then choose bitg with Control + Space
    (System Settings > Keyboard > Input Sources > + > bitg).
  EOS
end
