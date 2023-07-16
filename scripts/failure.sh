#! /bin/bash

echo
echo "Decrypting bin"
echo "Sending bin to amiitool...Done"
echo "Initializing NFC adapter"
echo "NFC reader: opened"
echo "***Scan tag***"
echo "Read UID: aa  aa  aa  aa  aa  aa  aa  "
echo
echo "Updating bin for new UID:"
echo "Replacing UID"
echo "Updating password"
echo "Writing magic bytes"
echo "Encrypting"
echo "Sending bin to amiitool...Done"
echo "Finished updating bin"
echo 
echo "Writing tag:"
echo "Writing encrypted bin:"
echo "Writing to 3: aa aa aa aa...Done"
echo "Writing to 4: aa aa aa aa...Failed"
echo

echo "Expected error" >&2
exit 1
