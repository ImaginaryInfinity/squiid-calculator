import requests
import xml.etree.ElementTree as ET
from datetime import datetime

def main():
	releases_data = requests.get('https://gitlab.com/api/v4/projects/44631396/releases').json()

	root = ET.Element("releases")
	for release in releases_data:
		release_elem = ET.SubElement(root, "release")
		release_elem.set("version", release["tag_name"])
		release_elem.set("date", 
			datetime.strftime(datetime.strptime(release["released_at"], '%Y-%m-%dT%H:%M:%S.%fZ'), '%Y-%m-%d')
		)
		
		url_elem = ET.SubElement(release_elem, "url")
		url_elem.text = release["_links"]["self"]

	tree = ET.ElementTree(root)
	tree.write("net.imaginaryinfinity.Squiid.releases.xml", encoding="utf-8", xml_declaration=True)


if __name__ == '__main__':
	main()