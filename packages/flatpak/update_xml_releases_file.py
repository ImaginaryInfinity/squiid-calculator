import requests
import xml.etree.ElementTree as ET
from datetime import datetime

def add_release(release, root):
	release_elem = ET.SubElement(root, "release")
	release_elem.set("version", release["tag_name"])
	release_elem.set("date", 
		datetime.strftime(datetime.strptime(release["released_at"], '%Y-%m-%dT%H:%M:%S.%fZ'), '%Y-%m-%d')
	)
	
	url_elem = ET.SubElement(release_elem, "url")
	url_elem.text = release["_links"]["self"]

def main():
	releases_data = requests.get('https://gitlab.com/api/v4/projects/44631396/releases').json()

	root = ET.Element("releases")
	next_version = input('Next version: ')
	add_release({
		'tag_name': next_version,
		'released_at': datetime.strftime(datetime.now(), '%Y-%m-%dT%H:%M:%S.%fZ'),
		'_links': {
			'self': f'https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/releases/{next_version}'
		}
	}, root)

	for release in releases_data:
		add_release(release, root)

	tree = ET.ElementTree(root)
	tree.write("net.imaginaryinfinity.Squiid.releases.xml", encoding="utf-8")


if __name__ == '__main__':
	main()