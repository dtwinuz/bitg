cask "bitg" do
  version "0.4.0"
  sha256 "57d86897600b6f66ee14cf7e4bd93b1adba5e55f9c33d836ee3e7700cc58352e"

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
